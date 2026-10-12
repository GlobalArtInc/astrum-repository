import React from 'react';
import { Badge } from '@radix-ui/themes';

import { dockerArtifactKindLabel, type DockerArtifactKind } from './dockerArtifactKind';

type BadgeColor = 'blue' | 'sky' | 'green' | 'amber' | 'purple' | 'cyan' | 'pink' | 'gray';

const KIND_COLORS: Record<DockerArtifactKind, BadgeColor> = {
  image: 'blue',
  helm: 'sky',
  signature: 'green',
  sbom: 'amber',
  attestation: 'purple',
  wasm: 'cyan',
  model: 'pink',
  other: 'gray',
};

export interface DockerArtifactBadgeProps {
  kind: DockerArtifactKind;
  size?: '1' | '2';
}

/**
 * A soft badge naming what a Docker manifest carries: image, Helm chart, signature, SBOM...
 */
export function DockerArtifactBadge({ kind, size = '1' }: DockerArtifactBadgeProps): JSX.Element {
  return (
    <Badge color={KIND_COLORS[kind]} variant="soft" size={size} radius="full" data-testid="docker-artifact-badge">
      {dockerArtifactKindLabel(kind)}
    </Badge>
  );
}
