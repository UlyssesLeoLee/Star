{{/*
Expand the name of the chart.
*/}}
{{- define "star.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "star.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{ include "star.selectorLabels" . }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "star.selectorLabels" -}}
app.kubernetes.io/name: {{ include "star.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
