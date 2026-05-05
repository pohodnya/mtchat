# MTChat Umbrella Chart Example

Template for organizations that need to deploy MTChat with **their own**
overrides, ingress rules, and extra Kubernetes resources — without forking
the upstream chart.

## Why an umbrella chart?

Copying `deploy/helm/mtchat/` into another repository works once, but every
upstream change has to be merged manually. An umbrella chart **depends on**
the published upstream chart and only contains the bits that are specific to
your environment:

- Per-environment values (`values-staging.yaml`, `values-prod.yaml`).
- Extra Kubernetes objects: `NetworkPolicy`, `ServiceMonitor`,
  `ExternalSecret`, organization-specific ingresses, etc.
- Glue with your platform: GitOps (ArgoCD/Flux), GitLab CI, Vault, etc.

To upgrade, you bump the dependency version, run `helm dependency update`,
and your CI takes care of the rest.

## Layout

```
your-repo/
├── Chart.yaml                       # depends on upstream mtchat
├── values.yaml                      # base overrides
├── values-prod.yaml.example         # production overlay (rename and edit)
├── templates/                       # YOUR extra resources only
│   ├── _helpers.tpl
│   └── network-policy.yaml
├── .helmignore
├── .gitlab-ci.yml.example
└── README.md
```

Copy this directory into your GitLab repo and adjust freely.

## Prerequisites

- Helm **3.8+** (OCI registry support).
- Authenticated access to `ghcr.io` for pulling the chart. Public artifacts
  allow anonymous pull, but a GitHub PAT with `read:packages` is the safest
  default for CI.

## Local workflow

```bash
# 1. One-time login to GHCR (only needed for private artifacts)
helm registry login ghcr.io -u <gh-username>

# 2. Resolve and download the upstream chart into ./charts/
helm dependency update

# 3. Render to inspect the output (no cluster contact)
helm template mtchat . -f values.yaml -f values-prod.yaml

# 4. Install / upgrade
helm upgrade --install mtchat . \
    --namespace mtchat --create-namespace \
    -f values.yaml \
    -f values-prod.yaml \
    --wait --timeout 5m
```

## Overriding upstream values

All upstream values live under the `mtchat:` key in `values.yaml` — that's
the dependency name from `Chart.yaml`. Anything you set there overrides the
default in `deploy/helm/mtchat/values.yaml` upstream.

```yaml
mtchat:
  api:
    replicaCount: 4
    extraEnv:
      - name: SENTRY_RELEASE
        value: "0.4.7"
  ingress:
    enabled: true
    className: nginx
```

See the upstream `values.yaml` for the full list of options.

## Adding your own resources

Drop any extra manifest into `templates/`. Helm renders these alongside the
upstream chart. Use the helper labels from `_helpers.tpl` for consistency
and `mtchat-deployment.apiSelector` to target upstream API pods.

A working `NetworkPolicy` example is included; toggle it via:

```yaml
networkPolicy:
  enabled: true
  allowedNamespace: ingress-nginx
```

## Patching upstream templates (rare)

If you need to **modify** an existing upstream template — say add a sidecar
container or change a `securityContext` field that has no values hook — use
a Helm post-renderer (typically `kustomize`). That is out of scope for this
template; see the [Helm documentation on post-renderers](https://helm.sh/docs/topics/advanced/#post-rendering).

The preferred fix is to add a values hook upstream so the patch becomes a
configuration change rather than a render-time mutation.

## Upgrading the upstream chart

1. Update both fields in `Chart.yaml`:
   ```yaml
   appVersion: "0.4.8"
   dependencies:
     - name: mtchat
       version: "0.4.8"
   ```
2. Run `helm dependency update` to refresh the `.tgz` in `charts/`.
3. Test with `helm template` and / or a staging deployment.
4. Commit the bumped `Chart.yaml` and the regenerated `Chart.lock`.

## Alternative: pull the tarball directly

If your CI cannot reach GHCR but can reach github.com, download the chart
tarball attached to the GitHub Release instead:

```bash
VERSION=0.4.7
curl -sL -o /tmp/mtchat.tgz \
  "https://github.com/pohodnya/mtchat/releases/download/v${VERSION}/mtchat-${VERSION}.tgz"
helm upgrade --install mtchat /tmp/mtchat.tgz -f values-prod.yaml
```

This skips the umbrella pattern altogether and is the simplest option when
you have **no** custom templates — only values overrides.
