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

By default, the compose file uses `pohodnya/mtchat:latest`. To build from source instead, uncomment the `build` section in `docker-compose.yml`.

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
  --set secret.JWT_SECRET="your-jwt-secret-min-32-chars" \
  --set secret.ADMIN_API_TOKEN="your-admin-api-token"
```

### With Ingress

```bash
helm install mtchat ./helm/mtchat \
  --set secret.JWT_SECRET="your-jwt-secret-min-32-chars" \
  --set secret.ADMIN_API_TOKEN="your-admin-api-token" \
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

More than one API replica requires Redis: WebSocket events and online status are
shared between replicas through it (see `docs/guide/architecture.md`). Use a
standalone Redis — Redis Cluster is not supported. Event delivery between
replicas is at-most-once; clients restore the state on reconnect. The chart
refuses to render more than one replica (`api.replicaCount`, or
`api.autoscaling.maxReplicas` with autoscaling) without Redis: set
`redis.enabled`, `redis.external.host`, `secret.REDIS_URL`, `existingSecret`
or `REDIS_URL` in `api.extraEnv`.

On SIGTERM an API pod releases the online status of its WebSocket connections
(at most 15 s) and publishes the queued events to the other replicas (at most
5 s). `api.terminationGracePeriodSeconds` (default 45) leaves room for that;
keep it above 20 s with a margin, or the pod may be killed before queued events go out.

#### Upgrading from 0.5.0 (rolling update)

Pods of 0.5.0 track online status only in `online:<user>` (set on ping,
removed when the user's last tab on that pod closes) and do not use the
`mtchat:ws:events` channel. While old and new pods run side by side:

- online status is temporarily inaccurate: an old pod may mark a user offline
  while they still have a tab on a new pod (and the reverse); it settles once
  every pod runs the new version and clients send their next ping;
- events are shared only between new pods: a user connected to an old pod
  misses events produced on other pods, as before the upgrade;
- rolling back to 0.5.0 is safe: the new keys (`presence:conns:<user>`) expire
  within 120 s and the channel is simply not used.

### Using a Values File

```bash
# Create a values override file
cat > my-values.yaml << EOF
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
