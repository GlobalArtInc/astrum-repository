

import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Box, Button, Card, Flex, Grid, Heading, Select, Table, Text, Theme, Tooltip } from '@radix-ui/themes';
import { Copy } from 'lucide-react';

import type { GADetail } from '../core';
import { parseGaCoordinates } from './detailHelpers';
import { getDependencySnippets } from './dependencySnippets/registry';
import { getFormatLabel } from './dependencySnippets/formatLabel';
import { trackSnippetCopy } from './dependencySnippets/trackSnippetCopy';
import type { DependencySnippet, SnippetComponentModel } from './dependencySnippets/types';
import { getUsageSnippets } from './dependencySnippets/usage';
import { dockerArtifactKind } from '../../../shared/DockerArtifact';
import { repositoryUrlOf } from '../../browse/useRepositoryInfo';

import { pageLanguage } from '../../../../../interface/plural';
import S from '../../../constants/pages/search/ComponentDetailStrings';
import '../../../shared/Usage/Usage.scss';

interface GAOverviewTabProps {
  detail: GADetail;
  selectedVersion: string | null;
  /**
   * Repository names holding the selected version.
   *
   * Passed in rather than read off `detail.repositories`, which is now always empty: it used to be
   * aggregated by walking every page of /v1/search, and that walk is gone (ASTRUM-54201 /
   * ASTRUM-54220). The caller sources this from the same per-version endpoint the Repositories tab
   * uses, so the "Repository" row and the snippet's registry URL agree with that tab.
   */
  repositories?: readonly string[];
  /** Registry of the first of those repositories, for formats whose commands name it (Docker). */
  registryUrl?: string;
  /** The selected version's most recent asset timestamp, or null if none carries one. */
  lastUpdated?: string | null;
  /**
   * Docker attributes of the selected version in the first of those repositories: a Docker repository holds any
   * OCI artifact, and a Helm chart or an SBOM is not pulled with docker.
   */
  dockerAttributes?: Readonly<Record<string, unknown>>;
}

/**
 * The commands for a Docker artifact that is no container image (Helm chart, signature, SBOM...), from the same
 * generator as the usage block of the repository browser; null for an image, which keeps the Docker snippets.
 */
function dockerArtifactSnippets(
  component: SnippetComponentModel,
  registryUrl: string | undefined,
  dockerAttributes: Readonly<Record<string, unknown>> | undefined,
): DependencySnippet[] | null {
  const kind = dockerArtifactKind(dockerAttributes ? { docker: dockerAttributes } : null);
  if (component.format !== 'docker' || !kind || kind === 'image' || !component.repositoryName) {
    return null;
  }
  return getUsageSnippets({
    format: 'docker',
    repositoryName: component.repositoryName,
    repositoryUrl: repositoryUrlOf(component.repositoryName),
    name: component.name,
    version: component.version,
    registryUrl: registryUrl ?? null,
    attributes: { docker: dockerAttributes },
  }).map((snippet) => ({
    displayName: snippet.label,
    snippetText: snippet.snippetText,
    description: snippet.description,
  }));
}

function formatDate(dateString: string | null | undefined): string {
  if (!dateString) return '—';
  try {
    const date = new Date(dateString);
    if (Number.isNaN(date.getTime())) return '—';
    return date.toLocaleDateString(pageLanguage(), { year: 'numeric', month: 'short', day: 'numeric' });
  } catch {
    return '—';
  }
}

/**
 * GAOverviewTab - Overview information for a GA.
 *
 * Renders per-format dependency snippets (ported from the Classic UI) and a Component Details
 * summary. The snippet set is chosen by repository format, so an npm component shows npm/Yarn
 * commands rather than a Maven pom.xml fragment. Formats with no generator show no snippet section.
 */
export function GAOverviewTab({
  detail,
  selectedVersion,
  repositories = [],
  registryUrl,
  lastUpdated = null,
  dockerAttributes,
}: GAOverviewTabProps) {
  const { gaId, format } = detail;
  const [copied, setCopied] = useState(false);
  const [selectedName, setSelectedName] = useState<string | null>(null);
  const copiedTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Clear any pending copied-reset timer when the tab unmounts so it can't fire afterwards.
  useEffect(
    () => () => {
      if (copiedTimerRef.current) {
        clearTimeout(copiedTimerRef.current);
      }
    },
    [],
  );

  // Navigating to a component of a different format exposes a different snippet set, so drop any
  // prior selection and fall back to that format's default rather than a stale (or absent) type.
  // Adjusting state during render (React's recommended pattern) avoids the one-frame flicker an
  // effect would leave when the old and new format share a snippet displayName.
  const [prevFormat, setPrevFormat] = useState(format);
  if (prevFormat !== format) {
    setPrevFormat(format);
    setSelectedName(null);
  }

  const { group, name: parsedName } = parseGaCoordinates(gaId);
  const name = parsedName || detail.displayName;
  // `??`, not `||`: '' is the valid selected version of a versionless format (raw), and there is
  // no `detail.versions[0]` to fall back to any more regardless.
  const ver = selectedVersion ?? '';

  const formatLabel = getFormatLabel(format);
  const repositoryName = repositories[0];
  const repositoryNames = repositories.join(', ');

  const snippets = useMemo(() => {
    const component: SnippetComponentModel = { format, group, name, version: ver, repositoryName };
    // The Overview is component-level, so there is no specific asset — matching Classic, which
    // emits no extension here. Synthesizing a jar extension for Maven would wrongly append @jar
    // to Gradle/PURL/Leiningen coordinates and break transitive resolution.
    return dockerArtifactSnippets(component, registryUrl, dockerAttributes)
      ?? getDependencySnippets(format, component, registryUrl ? { registryUrl } : undefined);
  }, [format, group, name, ver, repositoryName, registryUrl, dockerAttributes]);

  // Require a resolvable version: several generators (npm, pypi, maven, …) don't guard on an empty
  // version, so without this gate a version-less component would emit a truncated coordinate.
  const hasSnippets = snippets.length > 0 && !!ver;

  // The selected snippet, defaulting to the first one (mirrors the Classic combo behavior).
  const selectedSnippet = snippets.find((s) => s.displayName === selectedName) ?? snippets[0];

  const handleSelect = useCallback((displayName: string) => {
    setSelectedName(displayName);
    setCopied(false);
  }, []);

  const handleCopy = useCallback(
    (displayName: string, snippetText: string) => {
      // writeText can reject (permissions, insecure context); swallow it so it isn't an
      // unhandled rejection — the copy is best-effort and the analytics event still fires.
      navigator.clipboard.writeText(snippetText).catch(() => {});
      trackSnippetCopy(format, displayName);
      setCopied(true);
      if (copiedTimerRef.current) {
        clearTimeout(copiedTimerRef.current);
      }
      copiedTimerRef.current = setTimeout(() => setCopied(false), 2000);
    },
    [format],
  );

  const detailRows: Array<{ label: string; value: string; mono?: boolean }> = [
    { label: S.OVERVIEW.ROWS.LAST_UPDATED, value: formatDate(lastUpdated) },
    { label: S.OVERVIEW.ROWS.FORMAT, value: formatLabel },
    ...(group ? [{ label: S.OVERVIEW.ROWS.GROUP, value: group }] : []),
    ...(name ? [{ label: S.OVERVIEW.ROWS.NAME, value: name }] : []),
    ...(ver ? [{ label: S.OVERVIEW.ROWS.VERSION, value: ver }] : []),
    ...(repositoryNames ? [{ label: S.OVERVIEW.ROWS.REPOSITORY, value: repositoryNames, mono: true }] : []),
  ];

  return (
    <Flex direction="column" gap="6">
      <Grid columns={{ initial: '1', md: hasSnippets ? '2fr 1fr' : '1' }} gap="4">
        {hasSnippets && selectedSnippet && (
          <Card size="1">
            <Box p="4">
              <Heading size="4" mb="4">
                {S.OVERVIEW.DEPENDENCIES}
              </Heading>
              <Flex align="center" gap="2" mb="3">
                {/* Deliberately shows one snippet at a time behind this picker rather than stacking
                    them all: some formats (e.g. Maven/Java) expose many snippets, and stacking would
                    force the user to scroll past a long wall of code to reach the details below.
                    Radix Select keeps this consistent with every other dropdown in the Preview UI. */}
                <Select.Root value={selectedSnippet.displayName} onValueChange={handleSelect} size="2">
                  <Select.Trigger aria-label={S.OVERVIEW.SELECT_SNIPPET} style={{ flex: 1 }} />
                  <Select.Content>
                    {snippets.map((snippet) => (
                      <Select.Item key={snippet.displayName} value={snippet.displayName}>
                        {snippet.displayName}
                      </Select.Item>
                    ))}
                  </Select.Content>
                </Select.Root>
                <Tooltip content={copied ? S.OVERVIEW.COPIED : S.OVERVIEW.COPY}>
                  <Button
                    variant="soft"
                    color="blue"
                    size="2"
                    aria-label={S.OVERVIEW.COPY_SNIPPET(selectedSnippet.displayName)}
                    onClick={() => handleCopy(selectedSnippet.displayName, selectedSnippet.snippetText)}
                  >
                    <Copy size={14} />
                  </Button>
                </Tooltip>
              </Flex>
              {selectedSnippet.description && (
                <Text as="div" size="1" color="gray" mb="2">
                  {selectedSnippet.description}
                </Text>
              )}
              <Theme appearance="dark" hasBackground={false}>
                <div className="usage-block__code">
                  <pre className="usage-block__pre">
                    <code>{selectedSnippet.snippetText}</code>
                  </pre>
                </div>
              </Theme>
            </Box>
          </Card>
        )}

        <Card size="1">
          <Box p="4">
            <Heading size="4" mb="3">
              {S.OVERVIEW.COMPONENT_DETAILS}
            </Heading>
            <Table.Root variant="ghost" size="1">
              <Table.Body>
                {detailRows.map((row) => (
                  <Table.Row key={row.label}>
                    <Table.Cell>
                      <Text size="2" color="gray">
                        {row.label}
                      </Text>
                    </Table.Cell>
                    <Table.Cell>
                      <Text
                        size="2"
                        weight="medium"
                        style={row.mono ? { fontFamily: 'monospace', fontSize: '11px' } : undefined}
                      >
                        {row.value}
                      </Text>
                    </Table.Cell>
                  </Table.Row>
                ))}
              </Table.Body>
            </Table.Root>
          </Box>
        </Card>
      </Grid>
    </Flex>
  );
}

export default GAOverviewTab;
