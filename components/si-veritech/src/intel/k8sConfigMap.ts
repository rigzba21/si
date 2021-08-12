import {
  baseCheckQualifications,
  baseRunCommands,
  baseSyncResource,
} from "./k8sShared";

import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";

import {
  setProperty,
  setPropertyFromEntity,
  setPropertyFromProperty,
} from "./inferShared";

export function inferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const context = request.context;
  const entity = request.entity;

  setProperty({
    entity,
    toPath: ["metadata", "name"],
    value: entity.name,
  });

  setPropertyFromProperty({
    entity,
    fromPath: ["metadata", "name"],
    toPath: ["metadata", "labels", "app"],
  });

  // Do you have a k8s namespace? If so, set the namespace.
  setPropertyFromEntity({
    context,
    entityType: "k8sNamespace",
    fromPath: ["metadata", "name"],
    toEntity: entity,
    toPath: ["metadata", "namespace"],
  });

  // The template should have a namespace that matches the namespace of the
  // object we are deploying.
  setPropertyFromProperty({
    entity,
    fromPath: ["metadata", "namespace"],
    toPath: ["metadata", "namespace"],
  });

  return { entity };
}

export default {
  inferProperties,
  checkQualifications: baseCheckQualifications,
  runCommands: baseRunCommands,
  syncResource: baseSyncResource,
};
