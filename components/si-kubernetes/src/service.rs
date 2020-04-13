use crate::model::component::Component;
use crate::model::entity::{Entity, EntityEvent};
use crate::protobuf::deployment::{
    kubernetes_deployment_server, CreateEntityReply, CreateEntityRequest, EditPropObjectReply,
    EditPropObjectRequest, EditPropObjectYamlReply, EditPropObjectYamlRequest, GetComponentReply,
    GetComponentRequest, GetEntityReply, GetEntityRequest, ImplicitConstraint, ListComponentsReply,
    ListComponentsRequest, ListEntitiesReply, ListEntitiesRequest, ListEntityEventsReply,
    ListEntityEventsRequest, PickComponentReply, PickComponentRequest, SyncEntityReply,
    SyncEntityRequest,
};
use si_cea::service::prelude::*;

pub type Service = CeaService;

#[tonic::async_trait]
impl kubernetes_deployment_server::KubernetesDeployment for Service {
    async fn sync_entity(
        &self,
        request: TonicRequest<SyncEntityRequest>,
    ) -> TonicResult<SyncEntityReply> {
        gen_service_action!(self, request, "sync_entity", "sync", SyncEntityReply)
    }

    async fn list_entity_events(
        &self,
        mut request: TonicRequest<ListEntityEventsRequest>,
    ) -> TonicResult<ListEntityEventsReply> {
        gen_service_list!(self, request, "list_entity_events", EntityEvent)
    }

    async fn create_entity(
        &self,
        request: TonicRequest<CreateEntityRequest>,
    ) -> TonicResult<CreateEntityReply> {
        gen_service_create_entity!(self, request, "create_entity", CreateEntityReply)
    }

    async fn edit_prop_object(
        &self,
        request: TonicRequest<EditPropObjectRequest>,
    ) -> TonicResult<EditPropObjectReply> {
        gen_service_edit_prop!(
            self,
            request,
            edit_prop_object,
            EditPropObjectRequest,
            EditPropObjectReply
        )
    }

    async fn edit_prop_object_yaml(
        &self,
        request: TonicRequest<EditPropObjectYamlRequest>,
    ) -> TonicResult<EditPropObjectYamlReply> {
        gen_service_edit_prop!(
            self,
            request,
            edit_prop_object_yaml,
            EditPropObjectYamlRequest,
            EditPropObjectYamlReply
        )
    }

    async fn pick_component(
        &self,
        request: TonicRequest<PickComponentRequest>,
    ) -> TonicResult<PickComponentReply> {
        gen_service_pick_component!(self, request, "pick_component", PickComponentReply)
    }

    async fn list_components(
        &self,
        mut request: TonicRequest<ListComponentsRequest>,
    ) -> TonicResult<ListComponentsReply> {
        gen_service_list!(self, request, "list_components", Component)
    }

    async fn get_component(
        &self,
        request: TonicRequest<GetComponentRequest>,
    ) -> TonicResult<GetComponentReply> {
        gen_service_get!(
            self,
            request,
            "get_component",
            Component,
            component_id,
            GetComponentReply,
            component
        )
    }

    async fn list_entities(
        &self,
        mut request: TonicRequest<ListEntitiesRequest>,
    ) -> TonicResult<ListEntitiesReply> {
        gen_service_list!(self, request, "list_entities", Entity)
    }

    async fn get_entity(
        &self,
        request: TonicRequest<GetEntityRequest>,
    ) -> TonicResult<GetEntityReply> {
        gen_service_get!(
            self,
            request,
            "get_entity",
            Entity,
            entity_id,
            GetEntityReply,
            entity
        )
    }
}
