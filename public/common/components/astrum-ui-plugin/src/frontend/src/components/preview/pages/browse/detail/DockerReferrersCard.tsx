import React, { useEffect, useState } from 'react';
import { Card, Flex, Spinner, Text } from '@radix-ui/themes';
import { ChevronRight } from 'lucide-react';
import Axios from 'axios';

import { DockerArtifactBadge, dockerArtifactKind } from '../../../shared/DockerArtifact';
import { formatDateTime } from './detail.utils';
import UIStrings from '../../../constants/UIStrings';

const CREATED = 'org.opencontainers.image.created';

/**
 * A descriptor of the referrers API: a manifest that names the selected one as its subject.
 */
export interface DockerReferrer {
  mediaType?: string;
  digest: string;
  size?: number;
  artifactType?: string;
  annotations?: Record<string, string>;
}

export interface DockerReferrersCardProps {
  /** Repository holding the manifest */
  repositoryName: string;
  /** Image name, e.g. `charts/mychart` */
  image: string;
  /** Digest of the manifest */
  digest: string;
  /** Opens a referrer in the browse tree */
  onOpen?: (path: string) => void;
}

function strings() {
  return UIStrings.BROWSE.DETAIL.REFERRERS;
}

function shortDigest(digest: string): string {
  const colon = digest.indexOf(':');
  return colon < 0 ? digest.substring(0, 12) : `${digest.substring(0, colon + 1)}${digest.substring(colon + 1, colon + 13)}`;
}

/**
 * The artifacts attached to a Docker manifest (signatures, SBOMs, attestations...), as the referrers API of the
 * repository lists them, so they show up next to the image they belong to.
 */
export function DockerReferrersCard({ repositoryName, image, digest, onOpen }: DockerReferrersCardProps): JSX.Element {
  const [referrers, setReferrers] = useState<DockerReferrer[] | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setReferrers(null);
    setFailed(false);
    const url = `/repository/${encodeURIComponent(repositoryName)}/v2/${image}/referrers/${digest}`;
    Axios.get<{ manifests?: DockerReferrer[] }>(url, { headers: { Accept: 'application/vnd.oci.image.index.v1+json' } })
      .then((response) => {
        if (!cancelled) {
          setReferrers((response.data?.manifests ?? []).filter((referrer) => typeof referrer.digest === 'string'));
        }
      })
      .catch(() => {
        if (!cancelled) {
          setFailed(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [repositoryName, image, digest]);

  let body: React.ReactNode;
  if (failed) {
    body = <Text size="2" color="gray">{strings().ERROR}</Text>;
  } else if (referrers === null) {
    body = (
      <Flex align="center" gap="2" role="status">
        <Spinner size="1" />
        <Text size="2" color="gray">{strings().LOADING}</Text>
      </Flex>
    );
  } else if (referrers.length === 0) {
    body = <Text size="2" color="gray">{strings().EMPTY}</Text>;
  } else {
    body = (
      <ul className="detail-panel__referrers" data-testid="docker-referrers">
        {referrers.map((referrer) => {
          const kind = dockerArtifactKind({ docker: { artifactType: referrer.artifactType, mediaType: referrer.mediaType } })
            ?? 'other';
          const created = referrer.annotations?.[CREATED];
          return (
            <li key={referrer.digest}>
              <button
                type="button"
                className="detail-panel__referrer"
                aria-label={strings().OPEN(referrer.digest)}
                onClick={() => onOpen?.(`v2/${image}/manifests/${referrer.digest}`)}
              >
                <Flex direction="column" gap="1" minWidth="0" className="detail-panel__referrer-text">
                  <Flex align="center" gap="2" wrap="wrap">
                    <DockerArtifactBadge kind={kind} />
                    <Text size="1" color="gray" className="detail-panel__value--mono">{shortDigest(referrer.digest)}</Text>
                  </Flex>
                  {referrer.artifactType && (
                    <Text size="1" color="gray" className="detail-panel__referrer-type">{referrer.artifactType}</Text>
                  )}
                </Flex>
                <Flex align="center" gap="2">
                  {created && <Text size="1" color="gray">{formatDateTime(created)}</Text>}
                  <ChevronRight size={14} aria-hidden />
                </Flex>
              </button>
            </li>
          );
        })}
      </ul>
    );
  }

  return (
    <Card size="3" className="detail-panel__card" data-testid="detail-referrers">
      <Flex direction="column" gap="3">
        <Flex direction="column" gap="1">
          <Text as="div" size="3" weight="bold">{strings().TITLE}</Text>
          <Text as="p" size="2" color="gray">{strings().DESCRIPTION}</Text>
        </Flex>
        {body}
      </Flex>
    </Card>
  );
}

export default DockerReferrersCard;
