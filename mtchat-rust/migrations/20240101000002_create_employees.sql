-- Create employees table
CREATE TABLE employees (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    external_id UUID UNIQUE NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    has_access_to_any_dialog BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index on tenant_id for listing employees by tenant
CREATE INDEX idx_employees_tenant_id ON employees(tenant_id);

-- Index on external_id for lookups
CREATE INDEX idx_employees_external_id ON employees(external_id);
