{{/*
Expand the name of the chart.
*/}}
{{- define "mtchat.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "mtchat.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "mtchat.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "mtchat.labels" -}}
helm.sh/chart: {{ include "mtchat.chart" . }}
{{ include "mtchat.selectorLabels" . }}
app.kubernetes.io/version: {{ .Values.api.image.tag | default .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "mtchat.selectorLabels" -}}
app.kubernetes.io/name: {{ include "mtchat.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Service account name
*/}}
{{- define "mtchat.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "mtchat.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Secret name for application secrets
*/}}
{{- define "mtchat.secretName" -}}
{{- if .Values.secrets.existingSecret }}
{{- .Values.secrets.existingSecret }}
{{- else }}
{{- include "mtchat.fullname" . }}
{{- end }}
{{- end }}

{{/*
PostgreSQL host
*/}}
{{- define "mtchat.postgresql.host" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-postgresql" (include "mtchat.fullname" .) }}
{{- else }}
{{- .Values.postgresql.external.host }}
{{- end }}
{{- end }}

{{/*
PostgreSQL port
*/}}
{{- define "mtchat.postgresql.port" -}}
{{- if .Values.postgresql.enabled }}
{{- "5432" }}
{{- else }}
{{- .Values.postgresql.external.port | toString }}
{{- end }}
{{- end }}

{{/*
PostgreSQL database
*/}}
{{- define "mtchat.postgresql.database" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.database }}
{{- else }}
{{- .Values.postgresql.external.database }}
{{- end }}
{{- end }}

{{/*
PostgreSQL username
*/}}
{{- define "mtchat.postgresql.username" -}}
{{- if .Values.postgresql.enabled }}
{{- .Values.postgresql.auth.username }}
{{- else }}
{{- .Values.postgresql.external.username }}
{{- end }}
{{- end }}

{{/*
PostgreSQL secret name (for password)
*/}}
{{- define "mtchat.postgresql.secretName" -}}
{{- if .Values.postgresql.enabled }}
  {{- if .Values.postgresql.auth.existingSecret }}
  {{- .Values.postgresql.auth.existingSecret }}
  {{- else }}
  {{- printf "%s-postgresql" (include "mtchat.fullname" .) }}
  {{- end }}
{{- else }}
  {{- .Values.postgresql.external.existingSecret }}
{{- end }}
{{- end }}

{{/*
Redis host
*/}}
{{- define "mtchat.redis.host" -}}
{{- if .Values.redis.enabled }}
{{- printf "%s-redis" (include "mtchat.fullname" .) }}
{{- else }}
{{- .Values.redis.external.host }}
{{- end }}
{{- end }}

{{/*
Redis port
*/}}
{{- define "mtchat.redis.port" -}}
{{- if .Values.redis.enabled }}
{{- "6379" }}
{{- else }}
{{- .Values.redis.external.port | toString }}
{{- end }}
{{- end }}

{{/*
MinIO internal endpoint
*/}}
{{- define "mtchat.minio.endpoint" -}}
{{- if .Values.minio.enabled }}
{{- printf "http://%s-minio:9000" (include "mtchat.fullname" .) }}
{{- else }}
{{- .Values.minio.external.endpoint }}
{{- end }}
{{- end }}

{{/*
S3 bucket
*/}}
{{- define "mtchat.s3.bucket" -}}
{{- if .Values.minio.enabled }}
{{- .Values.minio.bucket }}
{{- else }}
{{- .Values.minio.external.bucket }}
{{- end }}
{{- end }}
