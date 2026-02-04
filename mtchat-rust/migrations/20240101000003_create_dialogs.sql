-- Create dialogs table
CREATE TABLE dialogs (
    id UUID PRIMARY KEY,
    chat_key VARCHAR(255) UNIQUE NOT NULL,
    tenant_a_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    tenant_b_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    context_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure tenant_a and tenant_b are different
    CONSTRAINT dialogs_different_tenants CHECK (tenant_a_id != tenant_b_id)
);

-- Index on chat_key for lookups
CREATE INDEX idx_dialogs_chat_key ON dialogs(chat_key);

-- Index on tenant IDs for listing dialogs by tenant
CREATE INDEX idx_dialogs_tenant_a ON dialogs(tenant_a_id);
CREATE INDEX idx_dialogs_tenant_b ON dialogs(tenant_b_id);

-- Index on context_id for finding context-specific dialogs
CREATE INDEX idx_dialogs_context_id ON dialogs(context_id) WHERE context_id IS NOT NULL;
