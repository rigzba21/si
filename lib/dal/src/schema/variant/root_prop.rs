//! This module contains (and is oriented around) the [`RootProp`]. This object is not persisted
//! to the database.

use strum::{AsRefStr, Display as EnumDisplay, EnumIter, EnumString};
use telemetry::prelude::*;

use crate::property_editor::schema::WidgetKind;
use crate::{
    schema::variant::{leaves::LeafKind, SchemaVariantResult},
    DalContext, Func, Prop, PropId, PropKind, ReconciliationPrototype,
    ReconciliationPrototypeContext, SchemaId, SchemaVariant, SchemaVariantId, StandardModel,
};

pub mod component_type;

/// This enum contains the subtree names for every direct child [`Prop`](crate::Prop) of
/// [`RootProp`](RootProp). Not all children will be of the same [`PropKind`](crate::PropKind).
#[remain::sorted]
#[derive(AsRefStr, EnumIter, EnumString, EnumDisplay)]
pub enum RootPropChild {
    /// Corresponds to the "/root/code" subtree.
    Code,
    /// Corresponds to the "/root/deleted_at" subtree.
    DeletedAt,
    /// Corresponds to the "/root/domain" subtree.
    Domain,
    /// Corresponds to the "/root/qualification" subtree.
    Qualification,
    /// Corresponds to the "/root/resource" subtree.
    Resource,
    /// Corresponds to the "/root/secrets" subtree.
    Secrets,
    /// Corresponds to the "/root/si" subtree.
    Si,
}

impl RootPropChild {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Si => "si",
            Self::Domain => "domain",
            Self::Resource => "resource",
            Self::Code => "code",
            Self::Qualification => "qualification",
            Self::DeletedAt => "deleted_at",
            Self::Secrets => "secrets",
        }
    }
}

/// This enum contains the subtree names for every direct child [`Prop`](crate::Prop) of "/root/si".
/// These [`Props`](crate::Prop) are available for _every_ [`SchemaVariant`](crate::SchemaVariant).
#[remain::sorted]
#[derive(Debug)]
pub enum SiPropChild {
    /// Corresponds to the "/root/si/Color" [`Prop`](crate::Prop).
    Color,
    /// Corresponds to the "/root/si/name" [`Prop`](crate::Prop).
    Name,
    /// Corresponds to the "/root/si/protected" [`Prop`](crate::Prop).
    Protected,
    /// Corresponds to the "/root/si/type" [`Prop`](crate::Prop).
    Type,
}

impl SiPropChild {
    /// Return the _case-sensitive_ name for the corresponding [`Prop`](crate::Prop).
    pub fn prop_name(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Protected => "protected",
            Self::Type => "type",
            Self::Color => "color",
        }
    }
}

/// Contains the root [`PropId`](crate::Prop) and its immediate children for a
/// [`SchemaVariant`](crate::SchemaVariant). These [`Props`](crate::Prop) are also those that
/// correspond to the "root" [`Props`](crate::Prop) on the [`ComponentView`](crate::ComponentView)
/// "properties" field.
#[derive(Debug, Copy, Clone)]
pub struct RootProp {
    /// The parent of the other [`Props`](crate::Prop) on [`self`](Self).
    pub prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to System Initiative metadata.
    pub si_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to the real world _model_.
    pub domain_prop_id: PropId,
    /// The parent of the resource [`Props`](crate::Prop) corresponding to the real world _resource_.
    pub resource_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) that are of secret value.
    pub secrets_prop_id: PropId,
    /// All information needed to populate the _model_ should be derived from this tree.
    pub resource_value_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to code generation
    /// [`Funcs`](crate::Func).
    pub code_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to qualification
    /// [`Funcs`](crate::Func).
    pub qualification_prop_id: PropId,
    /// The deleted_at prop on [`self`](Self).
    pub deleted_at_prop_id: PropId,
}

impl SchemaVariant {
    /// Create and set a [`RootProp`] for the [`SchemaVariant`].
    #[instrument(level = "debug", skip_all)]
    pub async fn create_and_set_root_prop(
        &mut self,
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaVariantResult<RootProp> {
        let root_prop =
            Prop::new_without_ui_optionals(ctx, "root", PropKind::Object, self.id, None).await?;
        let root_prop_id = *root_prop.id();
        self.set_root_prop_id(ctx, Some(root_prop_id)).await?;

        // FIXME(nick): we rely on ULID ordering for now, so the si prop tree creation has to come
        // before the domain prop tree creation. Once index maps for objects are added, this
        // can be moved back to its original location with the other prop tree creation methods.
        let si_prop_id = Self::setup_si(ctx, root_prop_id, schema_id, self.id).await?;

        let domain_prop = Prop::new_without_ui_optionals(
            ctx,
            "domain",
            PropKind::Object,
            self.id,
            Some(root_prop_id),
        )
        .await?;

        let secrets_prop_id = *Prop::new_without_ui_optionals(
            ctx,
            "secrets",
            PropKind::Object,
            self.id,
            Some(root_prop_id),
        )
        .await?
        .id();

        let resource_prop_id = Self::setup_resource(ctx, root_prop_id, self.id).await?;
        let resource_value_prop_id = Self::setup_resource_value(ctx, root_prop_id, self).await?;
        let code_prop_id = Self::setup_code(ctx, root_prop_id, self.id).await?;
        let qualification_prop_id = Self::setup_qualification(ctx, root_prop_id, self.id).await?;
        let deleted_at_prop_id = Self::setup_deleted_at(ctx, root_prop_id, self.id).await?;

        // Now that the structure is set up, we can populate default
        // AttributePrototypes & AttributeValues to be updated appropriately below.
        SchemaVariant::create_default_prototypes_and_values(ctx, self.id).await?;

        Ok(RootProp {
            prop_id: root_prop_id,
            si_prop_id,
            domain_prop_id: *domain_prop.id(),
            resource_value_prop_id,
            resource_prop_id,
            secrets_prop_id,
            code_prop_id,
            qualification_prop_id,
            deleted_at_prop_id,
        })
    }

    async fn insert_leaf_props(
        ctx: &DalContext,
        leaf_kind: LeafKind,
        root_prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<(PropId, PropId)> {
        let (leaf_prop_name, leaf_item_prop_name) = leaf_kind.prop_names();

        let mut leaf_prop = Prop::new_without_ui_optionals(
            ctx,
            leaf_prop_name,
            PropKind::Map,
            schema_variant_id,
            Some(root_prop_id),
        )
        .await?;
        leaf_prop.set_hidden(ctx, true).await?;

        let mut leaf_item_prop = Prop::new_without_ui_optionals(
            ctx,
            leaf_item_prop_name,
            PropKind::Object,
            schema_variant_id,
            Some(*leaf_prop.id()),
        )
        .await?;
        leaf_item_prop.set_hidden(ctx, true).await?;

        Ok((*leaf_prop.id(), *leaf_item_prop.id()))
    }

    async fn setup_si(
        ctx: &DalContext,
        root_prop_id: PropId,
        _schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<PropId> {
        let si_prop = Prop::new_without_ui_optionals(
            ctx,
            "si",
            PropKind::Object,
            schema_variant_id,
            Some(root_prop_id),
        )
        .await?;
        let si_prop_id = *si_prop.id();
        let _si_name_prop = Prop::new_without_ui_optionals(
            ctx,
            "name",
            PropKind::String,
            schema_variant_id,
            Some(si_prop_id),
        )
        .await?;

        // The protected prop ensures a component cannot be deleted in the configuration diagram.
        let _protected_prop = Prop::new_without_ui_optionals(
            ctx,
            "protected",
            PropKind::Boolean,
            schema_variant_id,
            Some(si_prop_id),
        )
        .await?;

        // The type prop controls the type of the configuration node. The default type can be
        // determined by the schema variant author. The widget options correspond to the component
        // type enumeration.
        Prop::new(
            ctx,
            "type",
            PropKind::String,
            schema_variant_id,
            Some(si_prop_id),
            Some((
                WidgetKind::Select,
                Some(serde_json::json!([
                    {
                        "label": "Component",
                        "value": "component",
                    },
                    {
                        "label": "Configuration Frame (down)",
                        "value": "configurationFrameDown",
                    },
                    {
                        "label": "Configuration Frame (up)",
                        "value": "configurationFrameUp",
                    },
                    {
                        "label": "Aggregation Frame",
                        "value": "aggregationFrame",
                    },
                ])),
            )),
            None,
            None,
        )
        .await?;

        // Override the schema variant color for nodes on the diagram.
        let mut color_prop = Prop::new_without_ui_optionals(
            ctx,
            "color",
            PropKind::String,
            schema_variant_id,
            Some(si_prop_id),
        )
        .await?;
        color_prop.set_widget_kind(ctx, WidgetKind::Color).await?;

        Ok(si_prop_id)
    }

    async fn setup_resource_value(
        ctx: &DalContext,
        root_prop_id: PropId,
        schema_variant: &mut SchemaVariant,
    ) -> SchemaVariantResult<PropId> {
        let schema_variant_id = *schema_variant.id();
        let mut resource_value_prop = Prop::new_without_ui_optionals(
            ctx,
            "resource_value",
            PropKind::Object,
            schema_variant_id,
            Some(root_prop_id),
        )
        .await?;
        resource_value_prop.set_hidden(ctx, true).await?;

        if let Some(reconciliation_func) =
            Func::find_by_attr(ctx, "name", &"si:defaultReconciliation")
                .await?
                .pop()
        {
            ReconciliationPrototype::upsert(
                ctx,
                *reconciliation_func.id(),
                "Reconciliation",
                ReconciliationPrototypeContext::new(*schema_variant.id()),
            )
            .await?;
        }

        SchemaVariant::create_default_prototypes_and_values(ctx, *schema_variant.id()).await?;
        SchemaVariant::create_implicit_internal_providers(ctx, *schema_variant.id()).await?;

        Ok(*resource_value_prop.id())
    }

    async fn setup_resource(
        ctx: &DalContext,
        root_prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<PropId> {
        let mut resource_prop = Prop::new_without_ui_optionals(
            ctx,
            "resource",
            PropKind::Object,
            schema_variant_id,
            Some(root_prop_id),
        )
        .await?;
        resource_prop.set_hidden(ctx, true).await?;
        let resource_prop_id = *resource_prop.id();

        let mut resource_status_prop = Prop::new_without_ui_optionals(
            ctx,
            "status",
            PropKind::String,
            schema_variant_id,
            Some(resource_prop_id),
        )
        .await?;
        resource_status_prop.set_hidden(ctx, true).await?;

        let mut resource_message_prop = Prop::new_without_ui_optionals(
            ctx,
            "message",
            PropKind::String,
            schema_variant_id,
            Some(resource_prop_id),
        )
        .await?;
        resource_message_prop.set_hidden(ctx, true).await?;

        let mut resource_logs_prop = Prop::new_without_ui_optionals(
            ctx,
            "logs",
            PropKind::Array,
            schema_variant_id,
            Some(resource_prop_id),
        )
        .await?;
        resource_logs_prop.set_hidden(ctx, true).await?;

        let mut resource_logs_log_prop = Prop::new_without_ui_optionals(
            ctx,
            "log",
            PropKind::String,
            schema_variant_id,
            Some(*resource_logs_prop.id()),
        )
        .await?;
        resource_logs_log_prop.set_hidden(ctx, true).await?;

        let mut resource_payload_prop = Prop::new_without_ui_optionals(
            ctx,
            "payload",
            PropKind::String,
            schema_variant_id,
            Some(resource_prop_id),
        )
        .await?;
        resource_payload_prop.set_hidden(ctx, true).await?;

        let mut resource_last_synced_prop = Prop::new_without_ui_optionals(
            ctx,
            "last_synced",
            PropKind::String,
            schema_variant_id,
            Some(resource_prop_id),
        )
        .await?;
        resource_last_synced_prop.set_hidden(ctx, true).await?;

        Ok(resource_prop_id)
    }

    async fn setup_code(
        ctx: &DalContext,
        root_prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<PropId> {
        let (code_map_prop_id, code_map_item_prop_id) = Self::insert_leaf_props(
            ctx,
            LeafKind::CodeGeneration,
            root_prop_id,
            schema_variant_id,
        )
        .await?;

        let mut child_code_prop = Prop::new_without_ui_optionals(
            ctx,
            "code",
            PropKind::String,
            schema_variant_id,
            Some(code_map_item_prop_id),
        )
        .await?;
        child_code_prop.set_hidden(ctx, true).await?;

        let mut child_message_prop = Prop::new_without_ui_optionals(
            ctx,
            "message",
            PropKind::String,
            schema_variant_id,
            Some(code_map_item_prop_id),
        )
        .await?;
        child_message_prop.set_hidden(ctx, true).await?;

        let mut child_format_prop = Prop::new_without_ui_optionals(
            ctx,
            "format",
            PropKind::String,
            schema_variant_id,
            Some(code_map_item_prop_id),
        )
        .await?;
        child_format_prop.set_hidden(ctx, true).await?;

        Ok(code_map_prop_id)
    }

    async fn setup_qualification(
        ctx: &DalContext,
        root_prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<PropId> {
        let (qualification_map_prop_id, qualification_map_item_prop_id) = Self::insert_leaf_props(
            ctx,
            LeafKind::Qualification,
            root_prop_id,
            schema_variant_id,
        )
        .await?;

        let mut child_qualified_prop = Prop::new_without_ui_optionals(
            ctx,
            "result",
            PropKind::String,
            schema_variant_id,
            Some(qualification_map_item_prop_id),
        )
        .await?;
        child_qualified_prop.set_hidden(ctx, true).await?;

        let mut child_message_prop = Prop::new_without_ui_optionals(
            ctx,
            "message",
            PropKind::String,
            schema_variant_id,
            Some(qualification_map_item_prop_id),
        )
        .await?;
        child_message_prop.set_hidden(ctx, true).await?;

        Ok(qualification_map_prop_id)
    }

    async fn setup_deleted_at(
        ctx: &DalContext,
        root_prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<PropId> {
        // This is a new prop that we will use to determine if we want to run a delete workflow
        let mut deleted_at = Prop::new_without_ui_optionals(
            ctx,
            "deleted_at",
            PropKind::String,
            schema_variant_id,
            Some(root_prop_id),
        )
        .await?;
        deleted_at.set_hidden(ctx, true).await?;

        Ok(*deleted_at.id())
    }
}
