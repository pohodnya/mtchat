# File Upload

MTChat supports file attachments via presigned S3 URLs. Files are uploaded directly from the client to S3/MinIO, bypassing the API server.

## Upload Flow

```
Client                    MTChat API              S3 / MinIO
  │                          │                        │
  │  1. POST /upload/presign │                        │
  │ ─────────────────────────>                        │
  │                          │                        │
  │  { upload_url, s3_key }  │                        │
  │ <─────────────────────────                        │
  │                          │                        │
  │  2. PUT upload_url (file body)                    │
  │ ──────────────────────────────────────────────────>
  │                          │                        │
  │  3. POST /messages       │                        │
  │  { attachments: [{s3_key}] }                      │
  │ ─────────────────────────>                        │
  │                          │                        │
```

1. Client requests a presigned upload URL from the API
2. Client uploads the file directly to S3 using the presigned URL
3. Client sends a message with the `s3_key` references

## Get Presigned Upload URL

```
POST /api/v1/upload/presign?user_id={uuid}
```

### Request Body

```json
{
  "dialog_id": "019481a2-...",
  "filename": "report.pdf",
  "content_type": "application/pdf",
  "size": 245760
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `dialog_id` | UUID | Yes | Dialog to upload the file for |
| `filename` | string | Yes | Original filename |
| `content_type` | string | Yes | MIME type of the file |
| `size` | integer | Yes | File size in bytes |

### Response

```json
{
  "data": {
    "upload_url": "https://s3.example.com/mtchat-attachments/dialogs/.../pending/...?X-Amz-...",
    "s3_key": "dialogs/019481a2-.../pending/019481d5-....pdf",
    "expires_in": 300
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `upload_url` | string | Presigned S3 URL for uploading (valid for `expires_in` seconds) |
| `s3_key` | string | S3 object key (pass this when sending the message) |
| `expires_in` | integer | URL expiry time in seconds (300 = 5 minutes) |

### Upload the File

Upload the file to S3 using the presigned URL:

```bash
curl -X PUT "${upload_url}" \
  -H "Content-Type: application/pdf" \
  --data-binary @report.pdf
```

### Attach to a Message

Include the `s3_key` when sending a message:

```json
{
  "content": "Here is the report",
  "attachments": [
    {
      "s3_key": "dialogs/019481a2-.../pending/019481d5-....pdf",
      "filename": "report.pdf",
      "content_type": "application/pdf",
      "size": 245760
    }
  ]
}
```

Messages can be sent with attachments only (empty content) or with both text and attachments.

## Get Attachment Download URL

Returns a presigned download URL for an attachment.

```
GET /api/v1/attachments/{id}/url?user_id={uuid}
```

### Response

```json
{
  "data": {
    "url": "https://s3.example.com/mtchat-attachments/dialogs/...?X-Amz-...",
    "thumbnail_url": null
  }
}
```

Download URLs are temporary and expire after a configurable period.

## Supported File Types

### Images

| Type | MIME |
|------|------|
| JPEG | `image/jpeg` |
| PNG | `image/png` |
| GIF | `image/gif` |
| WebP | `image/webp` |
| SVG | `image/svg+xml` |
| BMP | `image/bmp` |
| TIFF | `image/tiff` |

### Documents

| Type | MIME |
|------|------|
| PDF | `application/pdf` |
| Word | `application/msword`, `application/vnd.openxmlformats-officedocument.wordprocessingml.document` |
| Excel | `application/vnd.ms-excel`, `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` |
| PowerPoint | `application/vnd.ms-powerpoint`, `application/vnd.openxmlformats-officedocument.presentationml.presentation` |
| OpenDocument | `application/vnd.oasis.opendocument.*` |
| RTF | `application/rtf` |

### Text

`text/plain`, `text/csv`, `text/markdown`, `text/html`, `text/xml`, `application/json`

### Archives

`application/zip`, `application/x-rar-compressed`, `application/x-7z-compressed`, `application/gzip`, `application/x-tar`

### Audio

`audio/mpeg`, `audio/wav`, `audio/ogg`, `audio/mp4`

### Video

`video/mp4`, `video/webm`, `video/ogg`, `video/quicktime`

## Limits

| Limit | Value |
|-------|-------|
| Max file size | 100 MB |
| Max attachments per message | 10 |
| Upload URL expiry | 5 minutes |

## Built-in Viewer

The Vue SDK includes a built-in file viewer for:

- **Images** -- gallery view with keyboard navigation (arrows, Esc), zoom, and pan
- **PDFs** -- multi-page viewer with zoom, pan, and page navigation

Other file types display a download link with file icon, name, and size.

## Error Responses

| HTTP Status | Code | Description |
|-------------|------|-------------|
| 400 | `BAD_REQUEST` | File type not allowed or file too large |
| 404 | `NOT_FOUND` | Dialog or attachment not found |
| 500 | `INTERNAL_ERROR` | S3 not configured or S3 error |
