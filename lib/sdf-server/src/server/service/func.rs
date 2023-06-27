use crate::server::{impl_default_error_into_response, state::AppState};
use crate::service::func::get_func::GetFuncResponse;
use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use dal::{
    attribute::context::{AttributeContextBuilder, AttributeContextBuilderError},
    func::{
        argument::{FuncArgument, FuncArgumentError, FuncArgumentId, FuncArgumentKind},
        binding_return_value::FuncBindingReturnValueError,
    },
    prop_tree::PropTreeError,
    prototype_context::PrototypeContextError,
    schema::variant::SchemaVariantError,
    ActionKind, ActionPrototype, ActionPrototypeError, AttributeContext, AttributeContextError,
    AttributePrototype, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
    AttributePrototypeError, AttributePrototypeId, AttributeValueError, ComponentError,
    ComponentId, DalContext, ExternalProviderId, Func, FuncBackendKind, FuncBackendResponseType,
    FuncBindingError, FuncId, InternalProviderError, InternalProviderId, Prop, PropError, PropId,
    PropKind, PrototypeListForFuncError, SchemaVariantId, StandardModel, StandardModelError,
    TenancyError, TransactionsError, ValidationPrototype, ValidationPrototypeError, WsEventError,
};
use dal::{ExternalProviderError, FuncDescription, FuncDescriptionContents, LeafInputLocation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod create_func;
pub mod get_func;
pub mod list_funcs;
pub mod list_input_sources;
pub mod revert_func;
pub mod save_and_exec;
pub mod save_func;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncError {
    #[error("action func {0} assigned to multiple kinds")]
    ActionFuncMultipleKinds(FuncId),
    #[error("action kind missing on prototypes for action func {0}")]
    ActionKindMissing(FuncId),
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("That attribute is already set by the function named \"{0}\"")]
    AttributePrototypeAlreadySetByFunc(String),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype missing")]
    AttributePrototypeMissing,
    #[error("attribute prototype {0} is missing argument {1}")]
    AttributePrototypeMissingArgument(AttributePrototypeId, AttributePrototypeArgumentId),
    #[error("attribute prototype argument {0} is internal provider id")]
    AttributePrototypeMissingInternalProviderId(AttributePrototypeArgumentId),
    #[error("attribute prototype {0} is missing its prop {1}")]
    AttributePrototypeMissingProp(AttributePrototypeId, PropId),
    #[error("attribute prototype {0} has no PropId or ExternalProviderId")]
    AttributePrototypeMissingPropIdOrExternalProviderId(AttributePrototypeId),
    #[error("attribute prototype {0} schema is missing")]
    AttributePrototypeMissingSchema(AttributePrototypeId),
    #[error("attribute prototype {0} schema_variant is missing")]
    AttributePrototypeMissingSchemaVariant(AttributePrototypeId),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value missing")]
    AttributeValueMissing,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component missing schema variant")]
    ComponentMissingSchemaVariant(ComponentId),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("editing reconciliation functions is not implemented")]
    EditingReconciliationFuncsNotImplemented,
    #[error(transparent)]
    ExternalProvider(#[from] ExternalProviderError),
    #[error(transparent)]
    Func(#[from] dal::FuncError),
    #[error("func argument not found")]
    FuncArgNotFound,
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func argument already exists for that name")]
    FuncArgumentAlreadyExists,
    #[error("func argument {0} missing attribute prototype argument for prototype {1}")]
    FuncArgumentMissingPrototypeArgument(FuncArgumentId, AttributePrototypeId),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("func binding return value not found")]
    FuncBindingReturnValueMissing,
    #[error("func {0} cannot be converted to frontend variant")]
    FuncCannotBeTurnedIntoVariant(FuncId),
    // XXX: we will be able to remove this error once we make output sockets typed
    #[error("Cannot bind function to both an output socket and a prop")]
    FuncDestinationPropAndOutputSocket,
    #[error("cannot bind func to different prop kinds")]
    FuncDestinationPropKindMismatch,
    #[error("Function execution failed: {0}")]
    FuncExecutionFailed(String),
    #[error("Function execution failed: this function is not connected to any assets, and was not executed")]
    FuncExecutionFailedNoPrototypes,
    #[error("Function named \"{0}\" already exists in this changeset")]
    FuncNameExists(String),
    #[error("Function not found")]
    FuncNotFound,
    #[error("func is not revertible")]
    FuncNotRevertible,
    #[error("Cannot create that type of function")]
    FuncNotSupported,
    #[error("Function options are incompatible with variant")]
    FuncOptionsAndVariantMismatch,
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("Missing required options for creating a function")]
    MissingOptions,
    #[error("Function is read-only")]
    NotWritable,
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] Box<si_data_pg::PgPoolError>),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("prop for value not found")]
    PropNotFound,
    #[error("prop tree error: {0}")]
    PropTree(#[from] PropTreeError),
    #[error("prototype context error: {0}")]
    PrototypeContext(#[from] PrototypeContextError),
    #[error("prototype list for func error: {0}")]
    PrototypeListForFunc(#[from] PrototypeListForFuncError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant missing schema")]
    SchemaVariantMissingSchema(SchemaVariantId),
    #[error("Could not find schema variant for prop {0}")]
    SchemaVariantNotFoundForProp(PropId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
    #[error("unexpected func variant ({0:?}) creating attribute func")]
    UnexpectedFuncVariantCreatingAttributeFunc(FuncVariant),
    #[error("A validation already exists for that attribute")]
    ValidationAlreadyExists,
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),
    #[error("validation prototype schema is missing")]
    ValidationPrototypeMissingSchema,
    #[error("validation prototype {0} schema_variant is missing")]
    ValidationPrototypeMissingSchemaVariant(SchemaVariantId),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
}

impl From<si_data_pg::PgPoolError> for FuncError {
    fn from(value: si_data_pg::PgPoolError) -> Self {
        Self::PgPool(Box::new(value))
    }
}

pub type FuncResult<T> = Result<T, FuncError>;

impl_default_error_into_response!(FuncError);

// Variants don't map 1:1 onto FuncBackendKind, since some JsAttribute functions
// are a special case (Qualification, CodeGeneration etc)
#[remain::sorted]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum FuncVariant {
    Action,
    Attribute,
    CodeGeneration,
    Confirmation,
    Qualification,
    Reconciliation,
    Validation,
}

impl From<FuncVariant> for FuncBackendKind {
    fn from(value: FuncVariant) -> Self {
        match value {
            FuncVariant::Reconciliation => FuncBackendKind::JsReconciliation,
            FuncVariant::Action => FuncBackendKind::JsAction,
            FuncVariant::Validation => FuncBackendKind::JsValidation,
            FuncVariant::Attribute
            | FuncVariant::CodeGeneration
            | FuncVariant::Confirmation
            | FuncVariant::Qualification => FuncBackendKind::JsAttribute,
        }
    }
}

impl TryFrom<&Func> for FuncVariant {
    type Error = FuncError;

    fn try_from(func: &Func) -> Result<Self, Self::Error> {
        match (func.backend_kind(), func.backend_response_type()) {
            (FuncBackendKind::JsAttribute, response_type) => match response_type {
                FuncBackendResponseType::CodeGeneration => Ok(FuncVariant::CodeGeneration),
                FuncBackendResponseType::Qualification => Ok(FuncVariant::Qualification),
                FuncBackendResponseType::Confirmation => Ok(FuncVariant::Confirmation),
                _ => Ok(FuncVariant::Attribute),
            },
            (FuncBackendKind::JsReconciliation, _) => Ok(FuncVariant::Reconciliation),
            (FuncBackendKind::JsAction, _) => Ok(FuncVariant::Action),
            (FuncBackendKind::JsValidation, _) => Ok(FuncVariant::Validation),
            _ => Err(FuncError::FuncCannotBeTurnedIntoVariant(*func.id())),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeArgumentView {
    func_argument_id: FuncArgumentId,
    id: Option<AttributePrototypeArgumentId>,
    internal_provider_id: Option<InternalProviderId>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeView {
    id: AttributePrototypeId,
    component_id: Option<ComponentId>,
    prop_id: Option<PropId>,
    external_provider_id: Option<ExternalProviderId>,
    prototype_arguments: Vec<AttributePrototypeArgumentView>,
}

impl AttributePrototypeView {
    pub fn to_attribute_context(&self) -> FuncResult<AttributeContext> {
        let mut builder = AttributeContextBuilder::new();
        if let Some(component_id) = self.component_id {
            builder.set_component_id(component_id);
        }
        if let Some(prop_id) = self.prop_id {
            builder.set_prop_id(prop_id);
        }
        if let Some(external_provider_id) = self.external_provider_id {
            builder.set_external_provider_id(external_provider_id);
        }

        Ok(builder.to_context()?)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationPrototypeView {
    schema_variant_id: SchemaVariantId,
    prop_id: PropId,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncDescriptionView {
    schema_variant_id: SchemaVariantId,
    contents: FuncDescriptionContents,
}

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FuncAssociations {
    #[serde(rename_all = "camelCase")]
    Action {
        schema_variant_ids: Vec<SchemaVariantId>,
        kind: Option<ActionKind>,
    },
    #[serde(rename_all = "camelCase")]
    Attribute {
        prototypes: Vec<AttributePrototypeView>,
        arguments: Vec<FuncArgumentView>,
    },
    #[serde(rename_all = "camelCase")]
    CodeGeneration {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
        inputs: Vec<LeafInputLocation>,
    },
    #[serde(rename_all = "camelCase")]
    Confirmation {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
        descriptions: Vec<FuncDescriptionView>,
        inputs: Vec<LeafInputLocation>,
    },
    #[serde(rename_all = "camelCase")]
    Qualification {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
        inputs: Vec<LeafInputLocation>,
    },
    #[serde(rename_all = "camelCase")]
    SchemaVariantDefinitions {
        schema_variant_ids: Vec<SchemaVariantId>,
    },
    #[serde(rename_all = "camelCase")]
    Validation {
        prototypes: Vec<ValidationPrototypeView>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncArgumentView {
    pub id: FuncArgumentId,
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
}

async fn is_func_revertible(ctx: &DalContext, func: &Func) -> FuncResult<bool> {
    // refetch to get updated visibility
    let is_in_change_set = match Func::get_by_id(ctx, func.id()).await? {
        Some(func) => func.visibility().in_change_set(),
        None => return Ok(false),
    };
    // Clone a new ctx vith head visibility
    let ctx = ctx.clone_with_head();
    let head_func = Func::get_by_id(&ctx, func.id()).await?;

    Ok(head_func.is_some() && is_in_change_set)
}

async fn prototype_view_for_attribute_prototype(
    ctx: &DalContext,
    func_id: FuncId,
    proto: &AttributePrototype,
) -> FuncResult<AttributePrototypeView> {
    let prop_id = if proto.context.prop_id().is_some() {
        Some(proto.context.prop_id())
    } else {
        None
    };

    let external_provider_id = if proto.context.external_provider_id().is_some() {
        Some(proto.context.external_provider_id())
    } else {
        None
    };

    if prop_id.is_none() && external_provider_id.is_none() {
        return Err(FuncError::AttributePrototypeMissingPropIdOrExternalProviderId(*proto.id()));
    }

    let component_id = if proto.context.component_id().is_some() {
        Some(proto.context.component_id())
    } else {
        None
    };

    let prototype_arguments =
        FuncArgument::list_for_func_with_prototype_arguments(ctx, func_id, *proto.id())
            .await?
            .iter()
            .map(
                |(func_arg, maybe_proto_arg)| AttributePrototypeArgumentView {
                    func_argument_id: *func_arg.id(),
                    id: maybe_proto_arg.as_ref().map(|proto_arg| *proto_arg.id()),
                    internal_provider_id: maybe_proto_arg
                        .as_ref()
                        .map(|proto_arg| proto_arg.internal_provider_id()),
                },
            )
            .collect();

    Ok(AttributePrototypeView {
        id: *proto.id(),
        prop_id,
        component_id,
        external_provider_id,
        prototype_arguments,
    })
}

async fn action_prototypes_into_schema_variants_and_components(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncResult<(Option<ActionKind>, Vec<SchemaVariantId>)> {
    let mut variant_ids = vec![];
    let mut action_kind: Option<ActionKind> = None;

    for proto in ActionPrototype::find_for_func(ctx, func_id).await? {
        if let Some(action_kind) = &action_kind {
            if action_kind != proto.kind() {
                return Err(FuncError::ActionFuncMultipleKinds(func_id));
            }
        } else {
            action_kind = Some(*proto.kind());
        }

        if proto.schema_variant_id().is_some() {
            variant_ids.push(proto.schema_variant_id());
        }
    }

    if !variant_ids.is_empty() && action_kind.is_none() {
        return Err(FuncError::ActionKindMissing(func_id));
    }

    Ok((action_kind, variant_ids))
}

async fn attribute_prototypes_into_schema_variants_and_components(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncResult<(Vec<SchemaVariantId>, Vec<ComponentId>)> {
    let schema_variants_components =
        AttributePrototype::find_for_func_as_variant_and_component(ctx, func_id).await?;

    let mut schema_variant_ids = vec![];
    let mut component_ids = vec![];

    for (schema_variant_id, component_id) in schema_variants_components {
        if component_id == ComponentId::NONE {
            schema_variant_ids.push(schema_variant_id);
        } else {
            component_ids.push(component_id);
        }
    }

    Ok((schema_variant_ids, component_ids))
}

pub async fn func_description_views(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncResult<Vec<FuncDescriptionView>> {
    let mut views = vec![];

    for desc in FuncDescription::list_for_func(ctx, func_id).await? {
        views.push(FuncDescriptionView {
            schema_variant_id: *desc.schema_variant_id(),
            contents: desc.deserialized_contents()?,
        });
    }

    Ok(views)
}

pub async fn get_leaf_function_inputs(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncResult<Vec<LeafInputLocation>> {
    Ok(FuncArgument::list_for_func(ctx, func_id)
        .await?
        .iter()
        .filter_map(|arg| LeafInputLocation::maybe_from_arg_name(arg.name()))
        .collect())
}

pub async fn get_func_view(ctx: &DalContext, func: &Func) -> FuncResult<GetFuncResponse> {
    let arguments = FuncArgument::list_for_func(ctx, *func.id()).await?;

    let (associations, input_type) = match func.backend_kind() {
        FuncBackendKind::JsAttribute => {
            let input_type = compile_argument_types(&arguments);

            let associations = match func.backend_response_type() {
                FuncBackendResponseType::CodeGeneration => {
                    let (schema_variant_ids, component_ids) =
                        attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                            .await?;

                    Some(FuncAssociations::CodeGeneration {
                        schema_variant_ids,
                        component_ids,
                        inputs: get_leaf_function_inputs(ctx, *func.id()).await?,
                    })
                }
                FuncBackendResponseType::Confirmation => {
                    let (schema_variant_ids, component_ids) =
                        attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                            .await?;

                    let descriptions = func_description_views(ctx, *func.id()).await?;

                    Some(FuncAssociations::Confirmation {
                        schema_variant_ids,
                        component_ids,
                        descriptions,
                        inputs: get_leaf_function_inputs(ctx, *func.id()).await?,
                    })
                }
                FuncBackendResponseType::Qualification => {
                    let (schema_variant_ids, component_ids) =
                        attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                            .await?;

                    Some(FuncAssociations::Qualification {
                        schema_variant_ids,
                        component_ids,
                        inputs: get_leaf_function_inputs(ctx, *func.id()).await?,
                    })
                }
                _ => {
                    let protos = AttributePrototype::find_for_func(ctx, func.id()).await?;

                    let mut prototypes = Vec::with_capacity(protos.len());
                    for proto in &protos {
                        prototypes.push(
                            prototype_view_for_attribute_prototype(ctx, *func.id(), proto).await?,
                        );
                    }

                    Some(FuncAssociations::Attribute {
                        prototypes,
                        arguments: arguments
                            .iter()
                            .map(|arg| FuncArgumentView {
                                id: *arg.id(),
                                name: arg.name().to_owned(),
                                kind: arg.kind().to_owned(),
                                element_kind: arg.element_kind().cloned(),
                            })
                            .collect(),
                    })
                }
            };
            (associations, input_type)
        }
        FuncBackendKind::JsAction => {
            let (kind, schema_variant_ids) =
                action_prototypes_into_schema_variants_and_components(ctx, *func.id()).await?;

            let associations = Some(FuncAssociations::Action {
                schema_variant_ids,
                kind,
            });

            (associations, compile_action_types())
        }
        FuncBackendKind::JsReconciliation => {
            return Err(FuncError::EditingReconciliationFuncsNotImplemented);
        }
        FuncBackendKind::JsValidation => {
            let protos = ValidationPrototype::list_for_func(ctx, *func.id()).await?;
            let input_type = compile_validation_types(ctx, &protos).await?;

            let associations = Some(FuncAssociations::Validation {
                prototypes: protos
                    .iter()
                    .map(|proto| ValidationPrototypeView {
                        schema_variant_id: proto.context().schema_variant_id(),
                        prop_id: proto.context().prop_id(),
                    })
                    .collect(),
            });
            (associations, input_type)
        }
        _ => (None, String::new()),
    };

    let is_revertible = is_func_revertible(ctx, func).await?;
    let types = [
        compile_return_types(*func.backend_response_type()),
        &input_type,
        langjs_types(),
    ]
    .join("\n");

    Ok(GetFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        variant: func.try_into()?,
        display_name: func.display_name().map(Into::into),
        name: func.name().to_owned(),
        description: func.description().map(|d| d.to_owned()),
        code: func.code_plaintext()?,
        is_builtin: func.builtin(),
        is_revertible,
        associations,
        types,
    })
}

// TODO FIXME(paulo): cleanup code repetition

pub fn compile_return_types(ty: FuncBackendResponseType) -> &'static str {
    // TODO: avoid any, follow prop graph and build actual type
    // TODO: Could be generated automatically from some rust types, but which?
    match ty {
        FuncBackendResponseType::Boolean => "type Output = boolean | null;",
        FuncBackendResponseType::String => "type Output = string | null;",
        FuncBackendResponseType::Integer => "type Output = number | null;",
        FuncBackendResponseType::Qualification => {
            "interface Output {
  result: 'success' | 'warning' | 'failure';
  message?: string;
}"
        }
        FuncBackendResponseType::Confirmation => {
            "type ActionKind = 'create' | 'delete' | 'other' | 'refresh';
interface Output {
  success: boolean;
  recommendedActions: ActionKind[];
}"
        }
        FuncBackendResponseType::CodeGeneration => {
            "interface Output {
  format: string;
  code: string;
}"
        }
        FuncBackendResponseType::Validation => {
            "interface Output {
  valid: boolean;
  message: string;
}"
        }
        FuncBackendResponseType::Reconciliation => {
            "interface Output {
  updates: { [key: string]: unknown };
  actions: string[];
  message: string | null;
}"
        }
        FuncBackendResponseType::Action => {
            "interface Output {
    status: 'ok' | 'warning' | 'error';
    payload?: { [key: string]: unknown } | null;
    message?: string;
}"
        }
        FuncBackendResponseType::Json => "type Output = any;",
        // Note: there is no ts function returning those
        FuncBackendResponseType::Identity => "interface Output extends Input {}",
        FuncBackendResponseType::Array => "type Output = any[];",
        FuncBackendResponseType::Map => "type Output = any;",
        FuncBackendResponseType::Object => "type Output = any;",
        FuncBackendResponseType::Unset => "type Output = undefined | null;",
        FuncBackendResponseType::SchemaVariantDefinition => concat!(
            include_str!("./ts_types/asset_types.d.ts"),
            "\n",
            "type Output = any;"
        ),
    }
}

async fn compile_validation_types(
    ctx: &DalContext,
    prototypes: &[ValidationPrototype],
) -> FuncResult<String> {
    let mut input_fields = Vec::new();
    // TODO: avoid any, follow prop graph and build actual type
    for prototype in prototypes {
        let prop = Prop::get_by_id(ctx, &prototype.context().prop_id())
            .await?
            .ok_or(PropError::NotFound(
                prototype.context().prop_id(),
                *ctx.visibility(),
            ))?;
        let ty = match prop.kind() {
            PropKind::Boolean => "boolean",
            PropKind::Integer => "number",
            PropKind::String => "string",
            PropKind::Array => "any[]",
            PropKind::Object => "any",
            PropKind::Map => "any",
        };
        input_fields.push(ty);
    }
    if input_fields.is_empty() {
        Ok("type Input = never;".to_owned())
    } else {
        let variants = input_fields.join(" | ");
        let types = format!("type Input = {variants};");
        Ok(types)
    }
}

fn compile_argument_types(arguments: &[FuncArgument]) -> String {
    let mut input_fields = HashMap::new();
    // TODO: avoid any, follow prop graph and build actual type
    for argument in arguments {
        let ty = match argument.kind() {
            FuncArgumentKind::Boolean => "boolean",
            FuncArgumentKind::Integer => "number",
            FuncArgumentKind::String => "string",
            FuncArgumentKind::Array => "any[]",
            FuncArgumentKind::Object => "any",
            FuncArgumentKind::Map => "any",
            FuncArgumentKind::Any => "any",
        };
        input_fields.insert(argument.name().to_owned(), ty);
    }
    let mut types = "interface Input {\n".to_owned();
    for (name, ty) in &input_fields {
        types.push_str(&format!("  {name}: {ty} | null;\n"));
    }
    types.push('}');
    types
}

// TODO FIXME(paulo): arguments for command functions are provided through a js function, so we can't predict this, we should fix it so the types are predictable
// Right now all workflow functions are builtins and the user can't create new workflow functions, so we can trust that they all are providing the same argument
//
// TODO: build properties types from prop tree
// Note: ComponentKind::Credential is unused and the implementation is broken, so let's ignore it for now
fn compile_action_types() -> String {
    "interface Input {
    kind: 'standard';
    properties: any;
}"
    .to_owned()
}

// TODO: stop duplicating definition
// TODO: use execa types instead of any
// TODO: add os, fs and path types (possibly fetch but I think it comes with DOM)
fn langjs_types() -> &'static str {
    "declare namespace YAML {
    function stringify(obj: unknown): string;
}
    declare namespace siExec {
    async function waitUntilEnd(execaFile: string, execaArgs?: string[], execaOptions?: any): Promise<any>;
}"
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list_funcs", get(list_funcs::list_funcs))
        .route("/get_func", get(get_func::get_func))
        .route("/create_func", post(create_func::create_func))
        .route("/save_func", post(save_func::save_func))
        .route("/save_and_exec", post(save_and_exec::save_and_exec))
        .route("/revert_func", post(revert_func::revert_func))
        .route(
            "/list_input_sources",
            get(list_input_sources::list_input_sources),
        )
}
