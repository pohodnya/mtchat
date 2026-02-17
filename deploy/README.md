# MTChat Deployment

## Docker Compose (Production)

### Quick Start

```bash
cd deploy/
cp .env.example .env
# Edit .env - change ALL default passwords and secrets!
docker compose up -d
```

### Services

| Service | Description | Internal Port |
|---------|-------------|---------------|
| postgres | PostgreSQL 17 | 5432 |
| redis | Redis 7 (PubSub, presence, jobs) | 6379 |
| minio | S3-compatible file storage | 9000/9001 |
| minio-init | Bucket initialization (runs once) | - |
| api | MTChat backend | 8080 |

### Using a Pre-built Image

By default, the compose file uses `ghcr.io/pohodnya/mtchat-api:latest`. To build from source instead, uncomment the `build` section in `docker-compose.yml`.

### Development Mode

For local development with exposed ports and debug logging:

```bash
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d
```

This adds:
- All service ports exposed on `127.0.0.1`
- Debug-level logging
- Example frontend app on port 80
- Builds the API from source

### External Services

To use external PostgreSQL, Redis, or S3 instead of bundled services, remove the corresponding service from `docker-compose.yml` and update the environment variables in `.env`.

## Helm Chart (Kubernetes)

### Quick Start

```bash
helm install mtchat ./helm/mtchat \
  --set secrets.jwtSecret="your-jwt-secret-min-32-chars" \
  --set secrets.adminApiToken="your-admin-api-token"
```

### With Ingress

```bash
helm install mtchat ./helm/mtchat \
  --set secrets.jwtSecret="your-jwt-secret-min-32-chars" \
  --set secrets.adminApiToken="your-admin-api-token" \
  --set ingress.enabled=true \
  --set ingress.className=nginx \
  --set "ingress.hosts[0].host=chat.example.com" \
  --set "ingress.hosts[0].paths[0].path=/" \
  --set "ingress.hosts[0].paths[0].pathType=Prefix"
```

### With External Database

```bash
helm install mtchat ./helm/mtchat \
  --set postgresql.enabled=false \
  --set postgresql.external.host=my-db.example.com \
  --set postgresql.external.database=mtchat \
  --set postgresql.external.username=mtchat \
  --set postgresql.external.existingSecret=my-db-secret
```

### With External S3

```bash
helm install mtchat ./helm/mtchat \
  --set minio.enabled=false \
  --set minio.external.endpoint=https://s3.amazonaws.com \
  --set minio.external.publicEndpoint=https://s3.amazonaws.com \
  --set minio.external.bucket=my-mtchat-bucket \
  --set minio.external.region=us-east-1 \
  --set minio.external.existingSecret=my-s3-secret
```

### Autoscaling

```bash
helm install mtchat ./helm/mtchat \
  --set api.autoscaling.enabled=true \
  --set api.autoscaling.minReplicas=2 \
  --set api.autoscaling.maxReplicas=10
```

### Using a Values File

```bash
# Create a values override file
cat > my-values.yaml << EOF
secrets:
  existingSecret: mtchat-secrets

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/proxy-read-timeout: "86400"
  hosts:
    - host: chat.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: mtchat-tls
      hosts:
        - chat.example.com

postgresql:
  enabled: false
  external:
    host: my-rds-instance.amazonaws.com
    existingSecret: mtchat-db-secret

redis:
  enabled: false
  external:
    host: my-elasticache.amazonaws.com
    existingSecret: mtchat-redis-secret

minio:
  enabled: false
  external:
    endpoint: https://s3.amazonaws.com
    publicEndpoint: https://my-bucket.s3.amazonaws.com
    bucket: my-mtchat-bucket
    region: us-east-1
    existingSecret: mtchat-s3-secret

api:
  replicaCount: 3
  autoscaling:
    enabled: true
EOF

helm install mtchat ./helm/mtchat -f my-values.yaml
```

### Chart Structure

```
helm/mtchat/
  Chart.yaml              # Chart metadata
  values.yaml             # Default configuration
  .helmignore             # Files to ignore
  templates/
    _helpers.tpl           # Template helpers
    NOTES.txt              # Post-install notes
    secret.yaml            # Secrets (JWT, admin token, passwords)
    configmap.yaml         # Non-sensitive configuration
    serviceaccount.yaml    # Service account
    api-deployment.yaml    # API Deployment
    api-service.yaml       # API Service
    api-hpa.yaml           # Horizontal Pod Autoscaler
    ingress.yaml           # Ingress resource
    postgresql.yaml        # PostgreSQL StatefulSet + Service
    redis.yaml             # Redis StatefulSet + Service
    minio.yaml             # MinIO StatefulSet + Service + Init Job
```
