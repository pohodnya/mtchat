# Загрузка файлов

MTChat поддерживает файловые вложения через presigned S3 URL. Файлы загружаются напрямую с клиента в S3/MinIO, минуя API-сервер.

## Процесс загрузки

```
Клиент                       MTChat API              S3 / MinIO
  │                              │                        │
  │  1. POST /upload/presign     │                        │
  │ ─────────────────────────────>                        │
  │                              │                        │
  │  { upload_url, s3_key }      │                        │
  │ <─────────────────────────────                        │
  │                              │                        │
  │  2. PUT upload_url (файл)                             │
  │ ──────────────────────────────────────────────────────>
  │                              │                        │
  │  3. POST /messages           │                        │
  │  { attachments: [{s3_key}] } │                        │
  │ ─────────────────────────────>                        │
```

## Получение presigned URL

```
POST /api/v1/upload/presign?user_id={uuid}
```

```json
{
  "dialog_id": "019481a2-...",
  "filename": "отчёт.pdf",
  "content_type": "application/pdf",
  "size": 245760
}
```

### Ответ

```json
{
  "data": {
    "upload_url": "https://s3.example.com/...?X-Amz-...",
    "s3_key": "dialogs/019481a2-.../019481d5-....pdf",
    "expires_in": 300
  }
}
```

## Получение URL для скачивания

```
GET /api/v1/attachments/{id}/url?user_id={uuid}
```

### Ответ

```json
{
  "data": {
    "url": "https://s3.example.com/mtchat-attachments/dialogs/...?X-Amz-...",
    "thumbnail_url": null,
    "expires_in": 3600
  }
}
```

## Поддерживаемые типы файлов

- **Изображения:** JPEG, PNG, GIF, WebP, SVG, BMP, TIFF
- **Документы:** PDF, Word, Excel, PowerPoint, OpenDocument, RTF
- **Текст:** TXT, CSV, Markdown, HTML, XML, JSON
- **Архивы:** ZIP, RAR, 7z, GZIP, TAR
- **Аудио:** MP3, WAV, OGG, M4A
- **Видео:** MP4, WebM, OGG, MOV

## Ограничения

| Ограничение | Значение |
|-------------|----------|
| Макс. размер файла | 100 МБ |
| Макс. вложений на сообщение | 10 |
| Время жизни URL загрузки | 5 минут |

## Встроенный просмотрщик

Vue SDK включает просмотрщик для:

- **Изображения** -- галерея с навигацией клавишами, масштабированием и панорамированием
- **PDF** -- многостраничный просмотр с масштабированием и навигацией

Остальные типы файлов отображаются как ссылка для скачивания с иконкой, именем и размером.

## Ошибки

| HTTP статус | Код | Описание |
|-------------|-----|----------|
| 400 | `FILE_TOO_LARGE` | Размер файла вне допустимых лимитов |
| 400 | `UNSUPPORTED_FILE_TYPE` | MIME-тип файла не разрешён |
| 404 | `DIALOG_NOT_FOUND` | Диалог не существует |
| 404 | `ATTACHMENT_NOT_FOUND` | Вложение не существует |
| 500 | `INTERNAL_ERROR` | S3 не настроен или произошла ошибка S3 |
