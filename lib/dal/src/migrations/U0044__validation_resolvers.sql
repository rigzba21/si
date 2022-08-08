CREATE TABLE validation_resolvers
(
    pk                           bigserial PRIMARY KEY,
    id                           bigserial                NOT NULL,
    tenancy_universal            bool                     NOT NULL,
    tenancy_billing_account_ids  bigint[],
    tenancy_organization_ids     bigint[],
    tenancy_workspace_ids        bigint[],
    visibility_change_set_pk     bigint                   NOT NULL DEFAULT -1,
    visibility_deleted_at        timestamp with time zone,
    created_at                   timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                   timestamp with time zone NOT NULL DEFAULT NOW(),
    validation_prototype_id      bigint                   NOT NULL,
    attribute_value_id           bigint                   NOT NULL,
    func_id                      bigint                   NOT NULL,
    func_binding_id              bigint                   NOT NULL,
    func_binding_return_value_id bigint                   NOT NULL
);
SELECT standard_model_table_constraints_v1('validation_resolvers');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('validation_resolvers', 'model', 'validation_resolver', 'Validation Resolver');

CREATE OR REPLACE FUNCTION validation_resolver_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_validation_prototype_id bigint,
    this_attribute_value_id bigint,
    this_func_binding_id bigint,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record               tenancy_record_v1;
    this_visibility_record            visibility_record_v1;
    this_new_row                      validation_resolvers%ROWTYPE;
    this_func_id                      bigint;
    this_func_binding_return_value_id bigint;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    SELECT func_binding_return_value_id
    INTO STRICT this_func_binding_return_value_id
    FROM attribute_values
    WHERE id = this_attribute_value_id;

    SELECT belongs_to_id
    INTO STRICT this_func_id
    FROM func_binding_belongs_to_func
    WHERE object_id = this_func_binding_id;

    INSERT INTO validation_resolvers (tenancy_universal,
                                      tenancy_billing_account_ids,
                                      tenancy_organization_ids,
                                      tenancy_workspace_ids,
                                      visibility_change_set_pk,
                                      visibility_deleted_at,
                                      validation_prototype_id,
                                      attribute_value_id,
                                      func_id,
                                      func_binding_id,
                                      func_binding_return_value_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_validation_prototype_id,
            this_attribute_value_id,
            this_func_id,
            this_func_binding_id,
            this_func_binding_return_value_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
