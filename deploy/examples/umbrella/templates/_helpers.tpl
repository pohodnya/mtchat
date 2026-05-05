{{/*
Standard labels for resources owned by THIS umbrella chart.
Upstream mtchat resources keep their own labels — these only apply to
templates under ./templates/.
*/}}
{{- define "mtchat-deployment.labels" -}}
app.kubernetes.io/name: {{ .Chart.Name }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
{{- end -}}

{{/*
Selector labels matching upstream mtchat API pods.
Use this when targeting mtchat resources from your own templates
(NetworkPolicy, ServiceMonitor, etc).
*/}}
{{- define "mtchat-deployment.apiSelector" -}}
app.kubernetes.io/name: mtchat
app.kubernetes.io/component: api
{{- end -}}
