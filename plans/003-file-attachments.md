# План реализации загрузки файлов и изображений

**Статус**: 🔄 В процессе (Backend ✅, Frontend ✅, Thumbnails ⏳, Docker ⏳)

## Обзор

Реализация системы вложений для чата:
- Загрузка изображений и PDF-файлов в S3-совместимое хранилище (MinIO)
- Доступ к файлам только через presigned URLs
- Несколько вложений к одному сообщению
- Thumbnail-ы для изображений, стилизованные блоки для файлов
- Галерея для просмотра изображений (PrimeVue Galleria)
- Просмотр PDF через pdfjs-dist

---

## 1. Архитектура

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ FileUpload  │  │ MessageList │  │ Viewers                 │  │
│  │ Component   │  │ + Thumbnails│  │ - ImageGallery          │  │
│  │             │  │ + FileCards │  │ - PDFViewer             │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
└─────────┼────────────────┼─────────────────────┼────────────────┘
          │                │                     │
          │ 1. Get         │ 3. Get              │ 4. Fetch file
          │ presigned      │ messages with       │ via presigned
          │ upload URL     │ attachments         │ URL
          │                │                     │
          ▼                ▼                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                      MTChat Backend (Rust)                       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ Upload API      │  │ Messages API    │  │ Presign API     │  │
│  │ POST /upload    │  │ GET /messages   │  │ GET /files/{id} │  │
│  │ - presign URL   │  │ - with attach.  │  │ - presign URL   │  │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘  │
│           │                    │                    │           │
│           ▼                    ▼                    ▼           │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    PostgreSQL                             │   │
│  │  attachments: id, message_id, filename, content_type,     │   │
│  │               s3_key, size, width, height, thumbnail_key  │   │
│  └──────────────────────────────────────────────────────────┘   │
│           │                                                     │
│           ▼                                                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                 MinIO (S3-compatible)                     │   │
│  │  Bucket: mtchat-attachments                               │   │
│  │  Key: {dialog_id}/{message_id}/{uuid}.{ext}               │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Модель данных

### 2.1 Таблица `attachments`

```sql
CREATE TABLE attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,

    -- File info
    filename VARCHAR(255) NOT NULL,           -- Original filename
    content_type VARCHAR(100) NOT NULL,       -- MIME type
    size BIGINT NOT NULL,                     -- File size in bytes

    -- S3 storage (bucket определяется через S3_BUCKET env var)
    s3_key VARCHAR(500) NOT NULL UNIQUE,      -- S3 object key

    -- Image metadata (nullable for non-images)
    width INTEGER,                            -- Image width
    height INTEGER,                           -- Image height
    thumbnail_s3_key VARCHAR(500),            -- Thumbnail S3 key (for images)

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Indexes
    CONSTRAINT valid_size CHECK (size > 0 AND size <= 104857600) -- Max 100MB
);

CREATE INDEX idx_attachments_message ON attachments(message_id);
CREATE INDEX idx_attachments_content_type ON attachments(content_type);
```

> **Note:** `s3_bucket` не хранится в БД — используется единый bucket из переменной окружения `S3_BUCKET`.

### 2.2 Rust Domain Model

```rust
// src/domain/attachment.rs

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Attachment {
    pub id: Uuid,
    pub message_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub s3_key: String,              // bucket берётся из S3Service.bucket
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub thumbnail_s3_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub url: String,           // Presigned URL
    pub thumbnail_url: Option<String>, // Presigned thumbnail URL
}

pub enum AttachmentType {
    Image,
    Pdf,
    Other,
}

impl Attachment {
    pub fn attachment_type(&self) -> AttachmentType {
        if self.content_type.starts_with("image/") {
            AttachmentType::Image
        } else if self.content_type == "application/pdf" {
            AttachmentType::Pdf
        } else {
            AttachmentType::Other
        }
    }
}
```

---

## 3. Backend API

### 3.1 Upload Flow (Presigned URL)

```
Client                    Backend                     MinIO
   │                         │                          │
   │ 1. POST /upload/presign │                          │
   │    {filename, type}     │                          │
   │ ───────────────────────>│                          │
   │                         │ 2. Generate presigned    │
   │                         │    PUT URL               │
   │                         │ ────────────────────────>│
   │                         │<────────────────────────│
   │ 3. Return presigned URL │                          │
   │<─────────────────────── │                          │
   │                         │                          │
   │ 4. PUT file directly    │                          │
   │ ─────────────────────────────────────────────────>│
   │                         │                          │
   │ 5. POST /messages       │                          │
   │    {content, attachments: [{s3_key, ...}]}        │
   │ ───────────────────────>│                          │
   │                         │ 6. Verify file exists    │
   │                         │ ────────────────────────>│
   │                         │<────────────────────────│
   │                         │ 7. Create message +      │
   │                         │    attachments in DB     │
   │<─────────────────────── │                          │
```

### 3.2 API Endpoints

```rust
// Upload API
POST /api/v1/upload/presign
Request:
{
    "filename": "photo.jpg",
    "content_type": "image/jpeg",
    "size": 1024000
}
Response:
{
    "upload_url": "https://minio:9000/mtchat/...?X-Amz-Signature=...",
    "s3_key": "dialogs/{dialog_id}/pending/{uuid}.jpg",
    "expires_in": 300
}

// Send message with attachments
POST /api/v1/dialogs/{dialog_id}/messages
Request:
{
    "content": "Check out these files",
    "attachments": [
        {
            "s3_key": "dialogs/.../pending/abc123.jpg",
            "filename": "photo.jpg",
            "content_type": "image/jpeg",
            "size": 1024000
        }
    ]
}
Response:
{
    "data": {
        "id": "msg-uuid",
        "content": "Check out these files",
        "attachments": [
            {
                "id": "att-uuid",
                "filename": "photo.jpg",
                "content_type": "image/jpeg",
                "size": 1024000,
                "url": "https://...presigned...",
                "thumbnail_url": "https://...presigned..."
            }
        ]
    }
}

// Get presigned download URL
GET /api/v1/attachments/{id}/url
Response:
{
    "url": "https://minio:9000/...?X-Amz-Signature=...",
    "expires_in": 3600
}

// Get messages with attachments
GET /api/v1/dialogs/{dialog_id}/messages
Response includes attachments with presigned URLs
```

### 3.3 S3 Service (Rust)

```rust
// src/services/s3_service.rs

use aws_sdk_s3::{Client, presigning::PresigningConfig};

pub struct S3Service {
    client: Client,
    bucket: String,
    public_endpoint: String, // For presigned URLs accessible from browser
}

impl S3Service {
    pub async fn generate_upload_url(
        &self,
        key: &str,
        content_type: &str,
        expires_in: Duration,
    ) -> Result<String, S3Error> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(expires_in)
            .build()?;

        let presigned = self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await?;

        Ok(self.rewrite_url(presigned.uri()))
    }

    pub async fn generate_download_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, S3Error> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(expires_in)
            .build()?;

        let presigned = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await?;

        Ok(self.rewrite_url(presigned.uri()))
    }

    pub async fn object_exists(&self, key: &str) -> Result<bool, S3Error> {
        match self.client.head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) if e.is_not_found() => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), S3Error> {
        self.client.delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }

    // Rewrite internal MinIO URL to public URL
    fn rewrite_url(&self, url: &str) -> String {
        url.replace("http://minio:9000", &self.public_endpoint)
    }
}
```

### 3.4 Thumbnail Generation

Для изображений генерируем thumbnail при подтверждении загрузки:

```rust
// src/services/image_service.rs

use image::{GenericImageView, ImageFormat};

const THUMBNAIL_MAX_SIZE: u32 = 300;

pub struct ImageService {
    s3: Arc<S3Service>,
}

impl ImageService {
    pub async fn process_image(
        &self,
        s3_key: &str,
    ) -> Result<ImageMetadata, ImageError> {
        // Download original
        let data = self.s3.get_object(s3_key).await?;

        // Get dimensions
        let img = image::load_from_memory(&data)?;
        let (width, height) = img.dimensions();

        // Generate thumbnail
        let thumbnail = img.thumbnail(THUMBNAIL_MAX_SIZE, THUMBNAIL_MAX_SIZE);
        let mut thumb_data = Vec::new();
        thumbnail.write_to(&mut Cursor::new(&mut thumb_data), ImageFormat::WebP)?;

        // Upload thumbnail
        let thumb_key = s3_key.replace("/original/", "/thumbnails/");
        self.s3.put_object(&thumb_key, &thumb_data, "image/webp").await?;

        Ok(ImageMetadata {
            width: width as i32,
            height: height as i32,
            thumbnail_s3_key: Some(thumb_key),
        })
    }
}
```

---

## 4. Frontend Components

### 4.1 Структура файлов

```
mtchat-vue/src/
├── components/
│   ├── chat/
│   │   ├── MessageItem.vue        # Обновить для attachments
│   │   ├── MessageInput.vue       # Обновить для file upload
│   │   ├── AttachmentPreview.vue  # Preview before send
│   │   ├── AttachmentThumbnail.vue # Image thumbnail in message
│   │   ├── AttachmentFile.vue     # File card in message
│   │   └── AttachmentList.vue     # List of attachments
│   │
│   └── viewers/
│       ├── ImageGallery.vue       # PrimeVue Galleria wrapper
│       └── PDFViewer.vue          # pdfjs-dist viewer
│
├── composables/
│   ├── useFileUpload.ts           # Upload logic
│   └── useAttachments.ts          # Attachment state
│
└── services/
    └── uploadService.ts           # S3 presigned upload
```

### 4.2 TypeScript Types

```typescript
// types/attachment.ts

export interface Attachment {
  id: string
  filename: string
  contentType: string
  size: number
  width?: number
  height?: number
  url: string
  thumbnailUrl?: string
}

export interface PendingAttachment {
  id: string // temporary client-side ID
  file: File
  filename: string
  contentType: string
  size: number
  progress: number // 0-100
  status: 'pending' | 'uploading' | 'uploaded' | 'error'
  s3Key?: string
  previewUrl?: string // local blob URL for preview
  error?: string
}

export interface UploadPresignResponse {
  uploadUrl: string
  s3Key: string
  expiresIn: number
}

export type AttachmentType = 'image' | 'pdf' | 'file'

export function getAttachmentType(contentType: string): AttachmentType {
  if (contentType.startsWith('image/')) return 'image'
  if (contentType === 'application/pdf') return 'pdf'
  return 'file'
}
```

### 4.3 Upload Service

```typescript
// services/uploadService.ts

export class UploadService {
  constructor(private baseUrl: string, private getAuthHeaders: () => HeadersInit) {}

  async getPresignedUrl(
    dialogId: string,
    filename: string,
    contentType: string,
    size: number
  ): Promise<UploadPresignResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/upload/presign`, {
      method: 'POST',
      headers: {
        ...this.getAuthHeaders(),
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        dialog_id: dialogId,
        filename,
        content_type: contentType,
        size,
      }),
    })

    if (!response.ok) throw new Error('Failed to get upload URL')
    return response.json()
  }

  async uploadFile(
    uploadUrl: string,
    file: File,
    onProgress?: (progress: number) => void
  ): Promise<void> {
    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest()

      xhr.upload.addEventListener('progress', (e) => {
        if (e.lengthComputable && onProgress) {
          onProgress(Math.round((e.loaded / e.total) * 100))
        }
      })

      xhr.addEventListener('load', () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          resolve()
        } else {
          reject(new Error(`Upload failed: ${xhr.status}`))
        }
      })

      xhr.addEventListener('error', () => reject(new Error('Upload failed')))

      xhr.open('PUT', uploadUrl)
      xhr.setRequestHeader('Content-Type', file.type)
      xhr.send(file)
    })
  }
}
```

### 4.4 useFileUpload Composable

```typescript
// composables/useFileUpload.ts

import { ref, computed } from 'vue'
import { UploadService } from '../services/uploadService'

const MAX_FILE_SIZE = 100 * 1024 * 1024 // 100MB
const ALLOWED_TYPES = [
  'image/jpeg', 'image/png', 'image/gif', 'image/webp',
  'application/pdf'
]

export function useFileUpload(dialogId: Ref<string>, uploadService: UploadService) {
  const pendingAttachments = ref<PendingAttachment[]>([])
  const isUploading = computed(() =>
    pendingAttachments.value.some(a => a.status === 'uploading')
  )

  async function addFiles(files: FileList | File[]) {
    for (const file of Array.from(files)) {
      // Validate
      if (file.size > MAX_FILE_SIZE) {
        console.error(`File too large: ${file.name}`)
        continue
      }
      if (!ALLOWED_TYPES.includes(file.type)) {
        console.error(`File type not allowed: ${file.type}`)
        continue
      }

      const pending: PendingAttachment = {
        id: crypto.randomUUID(),
        file,
        filename: file.name,
        contentType: file.type,
        size: file.size,
        progress: 0,
        status: 'pending',
        previewUrl: file.type.startsWith('image/')
          ? URL.createObjectURL(file)
          : undefined,
      }

      pendingAttachments.value.push(pending)

      // Start upload
      uploadFile(pending)
    }
  }

  async function uploadFile(pending: PendingAttachment) {
    try {
      pending.status = 'uploading'

      // Get presigned URL
      const { uploadUrl, s3Key } = await uploadService.getPresignedUrl(
        dialogId.value,
        pending.filename,
        pending.contentType,
        pending.size
      )

      // Upload to S3
      await uploadService.uploadFile(uploadUrl, pending.file, (progress) => {
        pending.progress = progress
      })

      pending.s3Key = s3Key
      pending.status = 'uploaded'
      pending.progress = 100

    } catch (error) {
      pending.status = 'error'
      pending.error = error instanceof Error ? error.message : 'Upload failed'
    }
  }

  function removeAttachment(id: string) {
    const index = pendingAttachments.value.findIndex(a => a.id === id)
    if (index !== -1) {
      const attachment = pendingAttachments.value[index]
      if (attachment.previewUrl) {
        URL.revokeObjectURL(attachment.previewUrl)
      }
      pendingAttachments.value.splice(index, 1)
    }
  }

  function getUploadedAttachments(): AttachmentInput[] {
    return pendingAttachments.value
      .filter(a => a.status === 'uploaded' && a.s3Key)
      .map(a => ({
        s3_key: a.s3Key!,
        filename: a.filename,
        content_type: a.contentType,
        size: a.size,
      }))
  }

  function clearAll() {
    pendingAttachments.value.forEach(a => {
      if (a.previewUrl) URL.revokeObjectURL(a.previewUrl)
    })
    pendingAttachments.value = []
  }

  return {
    pendingAttachments,
    isUploading,
    addFiles,
    removeAttachment,
    getUploadedAttachments,
    clearAll,
  }
}
```

### 4.5 AttachmentPreview Component

```vue
<!-- components/chat/AttachmentPreview.vue -->
<template>
  <div class="attachment-preview">
    <div
      v-for="attachment in attachments"
      :key="attachment.id"
      class="preview-item"
      :class="{
        uploading: attachment.status === 'uploading',
        error: attachment.status === 'error'
      }"
    >
      <!-- Image preview -->
      <img
        v-if="attachment.previewUrl"
        :src="attachment.previewUrl"
        class="preview-image"
      />

      <!-- File icon -->
      <div v-else class="preview-file">
        <i :class="getFileIcon(attachment.contentType)" />
        <span class="filename">{{ attachment.filename }}</span>
      </div>

      <!-- Progress overlay -->
      <div v-if="attachment.status === 'uploading'" class="progress-overlay">
        <ProgressBar :value="attachment.progress" />
      </div>

      <!-- Error overlay -->
      <div v-if="attachment.status === 'error'" class="error-overlay">
        <i class="pi pi-exclamation-triangle" />
        <span>{{ attachment.error }}</span>
      </div>

      <!-- Remove button -->
      <button class="remove-btn" @click="$emit('remove', attachment.id)">
        <i class="pi pi-times" />
      </button>
    </div>
  </div>
</template>
```

### 4.6 Message с Attachments

```vue
<!-- components/chat/MessageItem.vue (updated) -->
<template>
  <div class="message">
    <div class="message-content">{{ message.content }}</div>

    <!-- Attachments -->
    <div v-if="message.attachments?.length" class="message-attachments">
      <!-- Images grid -->
      <div v-if="imageAttachments.length" class="attachments-images">
        <AttachmentThumbnail
          v-for="(att, index) in imageAttachments"
          :key="att.id"
          :attachment="att"
          @click="openGallery(index)"
        />
      </div>

      <!-- Files list -->
      <div v-if="fileAttachments.length" class="attachments-files">
        <AttachmentFile
          v-for="att in fileAttachments"
          :key="att.id"
          :attachment="att"
          @click="openFile(att)"
        />
      </div>
    </div>
  </div>
</template>
```

### 4.7 AttachmentThumbnail Component

```vue
<!-- components/chat/AttachmentThumbnail.vue -->
<template>
  <div class="attachment-thumbnail" @click="$emit('click')">
    <img
      :src="attachment.thumbnailUrl || attachment.url"
      :alt="attachment.filename"
      loading="lazy"
    />
    <div class="thumbnail-overlay">
      <i class="pi pi-search-plus" />
    </div>
  </div>
</template>

<style scoped>
.attachment-thumbnail {
  position: relative;
  width: 120px;
  height: 120px;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
}

.attachment-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.thumbnail-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s;
}

.attachment-thumbnail:hover .thumbnail-overlay {
  opacity: 1;
}

.thumbnail-overlay i {
  color: white;
  font-size: 24px;
}
</style>
```

### 4.8 AttachmentFile Component

```vue
<!-- components/chat/AttachmentFile.vue -->
<template>
  <div class="attachment-file" @click="$emit('click')">
    <div class="file-icon" :class="iconClass">
      <i :class="icon" />
    </div>
    <div class="file-info">
      <span class="file-name">{{ attachment.filename }}</span>
      <span class="file-size">{{ formatFileSize(attachment.size) }}</span>
    </div>
    <button class="download-btn" @click.stop="download">
      <i class="pi pi-download" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Attachment } from '../../types'

const props = defineProps<{
  attachment: Attachment
}>()

const icon = computed(() => {
  if (props.attachment.contentType === 'application/pdf') {
    return 'pi pi-file-pdf'
  }
  return 'pi pi-file'
})

const iconClass = computed(() => {
  if (props.attachment.contentType === 'application/pdf') {
    return 'pdf'
  }
  return 'generic'
})

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function download() {
  window.open(props.attachment.url, '_blank')
}
</script>

<style scoped>
.attachment-file {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.attachment-file:hover {
  background: rgba(255, 255, 255, 0.08);
}

.file-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.file-icon.pdf {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.file-icon.generic {
  background: rgba(79, 195, 247, 0.2);
  color: #4fc3f7;
}

.file-icon i {
  font-size: 20px;
}

.file-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.file-name {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  font-size: 12px;
  color: #888;
}

.download-btn {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: #888;
  cursor: pointer;
}

.download-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}
</style>
```

### 4.9 ImageGallery Component

```vue
<!-- components/viewers/ImageGallery.vue -->
<template>
  <Galleria
    v-model:visible="visible"
    v-model:activeIndex="activeIndex"
    :value="images"
    :numVisible="5"
    containerStyle="max-width: 90vw"
    :circular="true"
    :fullScreen="true"
    :showItemNavigators="true"
    :showThumbnails="false"
  >
    <template #item="{ item }">
      <img
        :src="item.url"
        :alt="item.filename"
        style="width: 100%; max-height: 90vh; object-fit: contain;"
      />
    </template>

    <template #caption="{ item }">
      <div class="gallery-caption">
        <span class="filename">{{ item.filename }}</span>
        <Button
          icon="pi pi-download"
          text
          rounded
          @click="download(item)"
        />
      </div>
    </template>
  </Galleria>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import Galleria from 'primevue/galleria'
import Button from 'primevue/button'
import type { Attachment } from '../../types'

const props = defineProps<{
  images: Attachment[]
  initialIndex?: number
  modelValue: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const visible = ref(props.modelValue)
const activeIndex = ref(props.initialIndex || 0)

watch(() => props.modelValue, (val) => {
  visible.value = val
  if (val && props.initialIndex !== undefined) {
    activeIndex.value = props.initialIndex
  }
})

watch(visible, (val) => {
  emit('update:modelValue', val)
})

function download(item: Attachment) {
  window.open(item.url, '_blank')
}
</script>

<style scoped>
.gallery-caption {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.6);
}

.filename {
  color: white;
  font-size: 14px;
}
</style>
```

### 4.10 PDFViewer Component

```vue
<!-- components/viewers/PDFViewer.vue -->
<template>
  <Dialog
    v-model:visible="visible"
    modal
    :header="filename"
    :style="{ width: '90vw', height: '90vh' }"
    :contentStyle="{ height: '100%', padding: 0 }"
  >
    <div class="pdf-viewer">
      <!-- Toolbar -->
      <div class="pdf-toolbar">
        <Button
          icon="pi pi-minus"
          text
          rounded
          @click="zoomOut"
          :disabled="scale <= 0.5"
        />
        <span class="zoom-level">{{ Math.round(scale * 100) }}%</span>
        <Button
          icon="pi pi-plus"
          text
          rounded
          @click="zoomIn"
          :disabled="scale >= 3"
        />
        <span class="page-info">
          Page {{ currentPage }} of {{ totalPages }}
        </span>
        <Button
          icon="pi pi-chevron-left"
          text
          rounded
          @click="prevPage"
          :disabled="currentPage <= 1"
        />
        <Button
          icon="pi pi-chevron-right"
          text
          rounded
          @click="nextPage"
          :disabled="currentPage >= totalPages"
        />
        <div class="spacer" />
        <Button
          icon="pi pi-download"
          text
          rounded
          @click="download"
        />
      </div>

      <!-- Canvas container -->
      <div class="pdf-container" ref="containerRef">
        <canvas ref="canvasRef" />
      </div>

      <!-- Loading -->
      <div v-if="loading" class="pdf-loading">
        <ProgressSpinner />
      </div>
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'
import * as pdfjsLib from 'pdfjs-dist'
import Dialog from 'primevue/dialog'
import Button from 'primevue/button'
import ProgressSpinner from 'primevue/progressspinner'

// Set worker
pdfjsLib.GlobalWorkerOptions.workerSrc =
  `https://cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjsLib.version}/pdf.worker.min.js`

const props = defineProps<{
  url: string
  filename: string
  modelValue: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const visible = ref(props.modelValue)
const loading = ref(true)
const canvasRef = ref<HTMLCanvasElement>()
const containerRef = ref<HTMLDivElement>()

const pdfDoc = ref<pdfjsLib.PDFDocumentProxy | null>(null)
const currentPage = ref(1)
const totalPages = ref(0)
const scale = ref(1.5)

watch(() => props.modelValue, async (val) => {
  visible.value = val
  if (val) {
    await loadPdf()
  }
})

watch(visible, (val) => {
  emit('update:modelValue', val)
})

async function loadPdf() {
  loading.value = true
  try {
    pdfDoc.value = await pdfjsLib.getDocument(props.url).promise
    totalPages.value = pdfDoc.value.numPages
    currentPage.value = 1
    await renderPage()
  } catch (error) {
    console.error('Failed to load PDF:', error)
  } finally {
    loading.value = false
  }
}

async function renderPage() {
  if (!pdfDoc.value || !canvasRef.value) return

  const page = await pdfDoc.value.getPage(currentPage.value)
  const viewport = page.getViewport({ scale: scale.value })

  const canvas = canvasRef.value
  const context = canvas.getContext('2d')!

  canvas.height = viewport.height
  canvas.width = viewport.width

  await page.render({
    canvasContext: context,
    viewport: viewport,
  }).promise
}

function prevPage() {
  if (currentPage.value > 1) {
    currentPage.value--
    renderPage()
  }
}

function nextPage() {
  if (currentPage.value < totalPages.value) {
    currentPage.value++
    renderPage()
  }
}

function zoomIn() {
  if (scale.value < 3) {
    scale.value += 0.25
    renderPage()
  }
}

function zoomOut() {
  if (scale.value > 0.5) {
    scale.value -= 0.25
    renderPage()
  }
}

function download() {
  window.open(props.url, '_blank')
}
</script>

<style scoped>
.pdf-viewer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1a1a2e;
}

.pdf-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.05);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.zoom-level,
.page-info {
  font-size: 14px;
  color: #e0e0e0;
  min-width: 60px;
  text-align: center;
}

.spacer {
  flex: 1;
}

.pdf-container {
  flex: 1;
  overflow: auto;
  display: flex;
  justify-content: center;
  padding: 20px;
}

.pdf-container canvas {
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.pdf-loading {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(26, 26, 46, 0.8);
}
</style>
```

---

## 5. Этапы реализации

### Этап 5.1: Backend - S3 Service и миграция ✅
**Файлы:** `migrations/`, `src/services/s3.rs`

- [x] Создать миграцию `20250204000001_update_attachments_v3.sql`
- [x] Добавить S3 SDK в Cargo.toml (`aws-sdk-s3`)
- [x] Реализовать `S3Service`:
  - `generate_upload_url()`
  - `generate_download_url()`
  - `object_exists()`
  - `delete_object()`
- [x] Добавить конфигурацию S3 в `main.rs` (AppState)
- [ ] Обновить docker-compose для MinIO bucket init

### Этап 5.2: Backend - Upload API ✅
**Файлы:** `src/api/upload.rs`, `src/domain/attachment.rs`

- [x] Создать domain model `Attachment`, `AttachmentResponse`, `AttachmentInput`
- [x] Создать `AttachmentRepository` с CRUD операциями
- [x] Реализовать `POST /api/v1/upload/presign`
- [x] Реализовать `GET /api/v1/attachments/{id}/url`
- [x] Добавить роуты в `main.rs`

### Этап 5.3: Backend - Messages с Attachments ✅
**Файлы:** `src/api/messages.rs`, `src/repositories/attachment_repo.rs`

- [x] Обновить `SendMessageRequest` для attachments
- [x] Обновить `MessageResponse` с attachments
- [x] Добавить создание attachments при отправке сообщения
- [x] Добавить presigned URLs при получении сообщений
- [x] Валидация: файл существует в S3 перед созданием

### Этап 5.4: Backend - Thumbnails
**Файлы:** `src/services/image_service.rs`

- [ ] Добавить `image` crate в Cargo.toml
- [ ] Реализовать `ImageService::process_image()`
- [ ] Генерация thumbnail при подтверждении загрузки
- [ ] WebP формат для thumbnails

### Этап 5.5: Frontend - Upload Service ✅
**Файлы:** `mtchat-vue/src/sdk/api.ts`, `composables/useFileUpload.ts`

- [x] Добавить методы в `MTChatApi`: `getPresignedUploadUrl()`, `uploadFile()`
- [x] Создать `useFileUpload` composable
- [x] Добавить типы для attachments в `types/index.ts`
- [x] Progress tracking через XHR

### Этап 5.6: Frontend - UI Components ✅
**Файлы:** `mtchat-vue/src/components/chat/`

- [x] Обновить `MTChat.vue` с file picker (кнопка прикрепления)
- [x] Создать `AttachmentPreview.vue`
- [x] Обновить `MTChat.vue` для отображения attachments в сообщениях
- [x] Создать `AttachmentThumbnail.vue`
- [x] Создать `AttachmentFile.vue`
- [x] Создать `AttachmentList.vue`

### Этап 5.7: Frontend - Viewers ✅
**Файлы:** `mtchat-vue/src/components/chat/`

- [x] Создать `ImageGallery.vue` (custom lightbox с навигацией)
- [x] PDF открывается в новой вкладке (без отдельного viewer)
- [x] Интеграция gallery в MTChat.vue

### Этап 5.8: WebSocket Events
**Файлы:** Backend и Frontend

- [ ] Обновить WebSocket message event с attachments
- [ ] Обновить `handleMessageNew` во frontend

---

## 6. Конфигурация MinIO

### docker-compose.yml (дополнение)

```yaml
services:
  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    volumes:
      - minio_data:/data
    ports:
      - "9000:9000"   # API
      - "9001:9001"   # Console
    healthcheck:
      test: ["CMD", "mc", "ready", "local"]
      interval: 5s
      timeout: 5s
      retries: 5

  minio-init:
    image: minio/mc
    depends_on:
      minio:
        condition: service_healthy
    entrypoint: >
      /bin/sh -c "
      mc alias set myminio http://minio:9000 minioadmin minioadmin;
      mc mb --ignore-existing myminio/mtchat-attachments;
      mc anonymous set none myminio/mtchat-attachments;
      exit 0;
      "
```

### Environment Variables (Backend)

```env
# S3 Configuration (обязательные)
S3_ACCESS_KEY_ID=minioadmin
S3_SECRET_ACCESS_KEY=minioadmin
S3_BUCKET=mtchat-attachments
S3_ENDPOINT=http://minio:9000
S3_REGION=us-east-1                    # для MinIO можно любой

# S3 Configuration (опциональные, есть дефолты)
S3_PUBLIC_ENDPOINT=http://localhost:9000  # default: S3_ENDPOINT
S3_PRESIGN_UPLOAD_EXPIRY=300              # default: 300 (5 мин)
S3_PRESIGN_DOWNLOAD_EXPIRY=3600           # default: 3600 (1 час)
```

---

## 7. Ограничения и валидация

| Параметр | Значение |
|----------|----------|
| Максимальный размер файла | 100 MB |
| Разрешённые типы | image/jpeg, image/png, image/gif, image/webp, application/pdf |
| Максимум файлов на сообщение | 10 |
| Thumbnail max size | 300x300 px |
| Thumbnail format | WebP |
| Presigned upload expiry | 5 минут |
| Presigned download expiry | 1 час |

---

## 8. Безопасность

1. **Presigned URLs** — файлы доступны только по подписанному URL с ограниченным временем жизни
2. **Валидация типов** — только разрешённые MIME-types
3. **Валидация размера** — проверка на backend перед созданием attachment
4. **Проверка существования** — файл должен существовать в S3 перед привязкой к сообщению
5. **Изоляция по диалогам** — S3 key включает dialog_id для организации
6. **Bucket policy** — приватный bucket, без публичного доступа

---

## 9. Верификация

После реализации проверить:

1. **Upload Flow:**
   - [ ] Получение presigned URL работает
   - [ ] Загрузка файла в S3 через presigned URL
   - [ ] Progress отображается корректно
   - [ ] Ошибки обрабатываются

2. **Messages:**
   - [ ] Сообщение с attachments создаётся
   - [ ] Attachments возвращаются с presigned URLs
   - [ ] Thumbnails генерируются для изображений
   - [ ] WebSocket доставляет сообщения с attachments

3. **Display:**
   - [ ] Изображения отображаются как thumbnails
   - [ ] PDF отображается как file card
   - [ ] Клик на thumbnail открывает галерею
   - [ ] Клик на PDF открывает viewer

4. **Viewers:**
   - [ ] Галерея работает (навигация, zoom)
   - [ ] PDF viewer работает (страницы, zoom)
   - [ ] Download работает

5. **Edge Cases:**
   - [ ] Файл > 100MB отклоняется
   - [ ] Неподдерживаемый тип отклоняется
   - [ ] Expired presigned URL обрабатывается
   - [ ] Несколько файлов в одном сообщении
