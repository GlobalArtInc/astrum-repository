

import React from 'react';
import {
  Box,
  Flex,
  Text,
  Card,
  Grid,
  Separator,
  Badge,
  Heading,
} from '@radix-ui/themes';
import {
  Database,
  Trash2,
  Route,
  HardDrive,
  Clock,
  Info,
} from 'lucide-react';
import { FormatBadge } from '../../../../../shared';
import { TypeBadge } from '../../repositories/TypeBadge';
import { SettingsLink } from './settingsLink';
import type {
  RepositoryProfileData,
  BlobStoreInfo,
  CleanupPolicyInfo,
  RoutingRuleInfo,
} from '../types';
import { ensureTrailingSlash } from '../../../../../../../utils/url';
import CommonStrings from '../../../../../constants/CommonStrings';
import RepositoryProfileStrings from '../../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import { formatDuration } from '../../../../../shared/utils/formatLocale';
import { pageLanguage } from '../../../../../../../interface/plural';

// =============================================================================
// Types
// =============================================================================

interface RepositoryTabProps {
  repository: RepositoryProfileData;
  blobStore: BlobStoreInfo | null;
  cleanupPolicies: CleanupPolicyInfo[];
  routingRule: RoutingRuleInfo | null;
}

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * The Astrum One page of the repository's blob store: its edit page when the type is known, the list
 * otherwise, or nothing when the repository has no blob store.
 */
export function blobStorePath(blobStoreName: string | undefined, blobStore: BlobStoreInfo | null): string | undefined {
  if (!blobStoreName) {
    return undefined;
  }
  if (blobStore?.typeId) {
    return `admin/repository/blobstores/${encodeURIComponent(blobStore.typeId)}/${encodeURIComponent(blobStoreName)}`;
  }
  return 'admin/repository/blobstores';
}

// =============================================================================
// Helper Components
// =============================================================================

interface ProfileSectionProps {
  title: string;
  icon: React.ElementType;
  editPath?: string;
  children: React.ReactNode;
}

function ProfileSection({ title, icon: Icon, editPath, children }: ProfileSectionProps): JSX.Element {
  return (
    <Card size="2" mb="4">
      <Flex direction="column" gap="3">
        <Flex align="center" justify="between">
          <Flex align="center" gap="2">
            <Icon size={18} color="var(--accent-9)" />
            <Heading size="3" style={{ fontSize: 'var(--font-size-4)' }}>{title}</Heading>
          </Flex>
          {editPath && (
            <SettingsLink previewPath={editPath} label={CommonStrings.ACTIONS.CONFIGURE} />
          )}
        </Flex>
        <Separator size="4" />
        <Box pt="1">
          {children}
        </Box>
      </Flex>
    </Card>
  );
}

interface ConfigRowProps {
  label: string;
  value: React.ReactNode;
  isCode?: boolean;
  isList?: boolean;
  listItems?: string[];
  fullWidth?: boolean;
}

function ConfigRow({ label, value, isCode, isList, listItems, fullWidth }: ConfigRowProps): JSX.Element | null {
  if (!isList && (value === undefined || value === null)) return null;

  let displayValue: React.ReactNode;

  if (isList && listItems) {
    displayValue = listItems.length > 0 ? (
      <Flex direction="column" gap="1">
        {listItems.map((item, i) => (
          <Text key={i} size="2" style={isCode ? { fontFamily: 'var(--code-font-family)' } : undefined}>
            {item}
          </Text>
        ))}
      </Flex>
    ) : <Text size="2" color="gray">{RepositoryProfileStrings.REPOSITORY_TAB.NONE}</Text>;
  } else if (typeof value === 'boolean') {
    displayValue = (
      <Badge color={value ? 'green' : 'gray'} variant="soft">
        {value ? RepositoryProfileStrings.REPOSITORY_TAB.ENABLED : RepositoryProfileStrings.REPOSITORY_TAB.DISABLED}
      </Badge>
    );
  } else if (isCode) {
    displayValue = (
      <Text size="2" style={{ wordBreak: 'break-all', fontFamily: 'var(--code-font-family)', color: 'var(--accent-11)' }}>
        {String(value)}
      </Text>
    );
  } else {
    displayValue = React.isValidElement(value) ? value : (
      <Text size="2" weight="medium">
        {String(value)}
      </Text>
    );
  }

  return (
    <Box mb="3" style={fullWidth ? { gridColumn: 'span 2' } : undefined}>
      <Box mb="1">
        <Text size="1" color="gray" weight="bold" style={{ textTransform: 'uppercase', letterSpacing: '0.05em' }}>
          {label}
        </Text>
      </Box>
      {displayValue}
    </Box>
  );
}

// =============================================================================
// Helper Functions
// =============================================================================

function formatBytes(bytes: number | undefined): string {
  if (bytes === undefined || bytes === null) return '—';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let unitIndex = 0;
  let size = bytes;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`;
}

// =============================================================================
// Component
// =============================================================================

export function RepositoryTab({
  repository,
  blobStore,
  cleanupPolicies,
  routingRule,
}: RepositoryTabProps): JSX.Element {
  const attrs = repository.attributes || {};
  const storage = attrs.storage;
  const proxy = attrs.proxy;
  const negativeCache = attrs.negativeCache;
  const httpClient = attrs.httpClient;
  const group = attrs.group;
  const cleanup = attrs.cleanup;
  const _component = attrs.component;
  const maven = attrs.maven;
  const docker = attrs.docker;
  const _dockerProxy = attrs.dockerProxy;
  const npm = attrs.npm;
  const nugetProxy = attrs.nugetProxy;
  const apt = attrs.apt;
  const _aptSigning = attrs.aptSigning;
  const yum = attrs.yum;
  const raw = attrs.raw;
  const replication = attrs.replication;
  const _cocoapods = attrs.cocoapods;
  const _conan = attrs.conan;
  const _conda = attrs.conda;
  const _goConfig = attrs.go;
  const _gitlfs = attrs.gitlfs;
  const _helm = attrs.helm;
  const _p2 = attrs.p2;
  const _r = attrs.r;
  const _rubygems = attrs.rubygems;

  const isProxy = repository.type === 'proxy';
  const isHosted = repository.type === 'hosted';
  const isGroup = repository.type === 'group';

  return (
    <Box>
      <ProfileSection
        title={RepositoryProfileStrings.REPOSITORY_TAB.REPOSITORY_CONFIGURATION}
        icon={Database}
        editPath={`admin/repository/repositories/${encodeURIComponent(repository.name)}`}
      >
        <Grid columns="2" gapX="6" gapY="2">
          <Box style={{ gridColumn: 'span 2' }} mb="2">
            <Text size="2" weight="bold" color="accent">{RepositoryProfileStrings.REPOSITORY_TAB.GENERAL}</Text>
          </Box>
          <ConfigRow label={CommonStrings.LABELS.NAME} value={repository.name} />
          <ConfigRow label={CommonStrings.LABELS.FORMAT} value={<FormatBadge format={repository.format} />} />
          <ConfigRow label={CommonStrings.LABELS.TYPE} value={<TypeBadge type={repository.type as any} />} />
          <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.RECIPE} value={repository.recipe} />
          <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.ONLINE} value={repository.online} />
          <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.URL} value={ensureTrailingSlash(repository.url)} isCode fullWidth />
          
          {repository.routingRuleId && (
            <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.ROUTING_RULE} value={repository.routingRuleId} />
          )}

          <Box style={{ gridColumn: 'span 2' }} mt="4" mb="2">
            <Text size="2" weight="bold" color="accent">{RepositoryProfileStrings.REPOSITORY_TAB.STORAGE_CLEANUP}</Text>
          </Box>
          <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.BLOB_STORE} value={storage?.blobStoreName} />
          <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.STRICT_CONTENT_TYPE_VALIDATION} value={storage?.strictContentTypeValidation} />
          {isHosted && <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.WRITE_POLICY} value={storage?.writePolicy} />}
          
          <ConfigRow
            label={RepositoryProfileStrings.REPOSITORY_TAB.CLEANUP_POLICIES}
            value={undefined}
            isList
            listItems={cleanup?.policyNames || []}
          />

          {isProxy && (
            <>
              <Box style={{ gridColumn: 'span 2' }} mt="4" mb="2">
                <Text size="2" weight="bold" color="accent">{RepositoryProfileStrings.REPOSITORY_TAB.PROXY_CONFIGURATION}</Text>
              </Box>
              <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.REMOTE_STORAGE} value={proxy?.remoteUrl} isCode fullWidth />
              <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.CONTENT_MAX_AGE} value={proxy?.contentMaxAge !== undefined ? formatDuration(proxy.contentMaxAge, 'minute') : undefined} />
              <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.METADATA_MAX_AGE} value={proxy?.metadataMaxAge !== undefined ? formatDuration(proxy.metadataMaxAge, 'minute') : undefined} />

              {negativeCache && (
                <>
                  <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.NEGATIVE_CACHE_ENABLED} value={negativeCache.enabled} />
                  <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.NEGATIVE_CACHE_TTL} value={negativeCache.timeToLive !== undefined ? formatDuration(negativeCache.timeToLive, 'minute') : undefined} />
                </>
              )}

              {httpClient && (
                <>
                  <Box style={{ gridColumn: 'span 2' }} mt="2" mb="2">
                    <Text size="1" weight="bold" color="gray">{RepositoryProfileStrings.REPOSITORY_TAB.HTTP_CLIENT}</Text>
                  </Box>
                  <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.BLOCKED} value={httpClient.blocked} />
                  <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.AUTO_BLOCK} value={httpClient.autoBlock} />
                  {httpClient.connection && (
                    <>
                      <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.RETRIES} value={httpClient.connection.retries} />
                      <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.TIMEOUT} value={httpClient.connection.timeout !== undefined ? formatDuration(httpClient.connection.timeout, 'second') : undefined} />
                      <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.USE_TRUST_STORE} value={httpClient.connection.useTrustStore} />
                    </>
                  )}
                  {httpClient.authentication && (
                    <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.AUTH_TYPE} value={httpClient.authentication.type} />
                  )}
                </>
              )}
            </>
          )}

          {isGroup && group && (
            <>
              <Box style={{ gridColumn: 'span 2' }} mt="4" mb="2">
                <Text size="2" weight="bold" color="accent">{RepositoryProfileStrings.REPOSITORY_TAB.GROUP_SETTINGS}</Text>
              </Box>
              <ConfigRow
                label={RepositoryProfileStrings.REPOSITORY_TAB.MEMBER_REPOSITORIES}
                value={undefined}
                isList
                listItems={group.memberNames || []}
                fullWidth
              />
              <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.WRITABLE_MEMBER} value={group.writableMember || RepositoryProfileStrings.REPOSITORY_TAB.NONE} />
            </>
          )}

          {/* Format Specific Settings */}
          {(maven || docker || npm || nugetProxy || apt || yum || raw || replication) && (
            <Box style={{ gridColumn: 'span 2' }} mt="4" mb="2">
              <Text size="2" weight="bold" color="accent">{RepositoryProfileStrings.REPOSITORY_TAB.FORMAT_SETTINGS(repository.format)}</Text>
            </Box>
          )}
          
          {maven && (
            <>
              <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.VERSION_POLICY} value={maven.versionPolicy} />
              <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.LAYOUT_POLICY} value={maven.layoutPolicy} />
            </>
          )}
          
          {docker && (
            <>
              <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.HTTP_PORT} value={docker.httpPort} />
              <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.HTTPS_PORT} value={docker.httpsPort} />
            </>
          )}

          {npm && (
            <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.REMOVE_QUARANTINED} value={npm.removeQuarantined} />
          )}
        </Grid>
      </ProfileSection>

      <ProfileSection
        title={RepositoryProfileStrings.REPOSITORY_TAB.BLOB_STORE_2(storage?.blobStoreName || '—')}
        icon={HardDrive}
        editPath={blobStorePath(storage?.blobStoreName, blobStore)}
      >
        {blobStore ? (
          <Grid columns="3" gap="4">
            <ConfigRow label={CommonStrings.LABELS.NAME} value={blobStore.name} />
            <ConfigRow label={CommonStrings.LABELS.TYPE} value={blobStore.type} />
            <ConfigRow label={CommonStrings.LABELS.STATUS} value={blobStore.unavailable ? <Badge color="red">{RepositoryProfileStrings.REPOSITORY_TAB.UNAVAILABLE}</Badge> : <Badge color="green">{RepositoryProfileStrings.REPOSITORY_TAB.AVAILABLE}</Badge>} />
            <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.TOTAL_SIZE} value={formatBytes(blobStore.totalSizeInBytes ?? 0)} />
            <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.AVAILABLE_SPACE} value={formatBytes(blobStore.availableSpaceInBytes ?? 0)} />
            <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.BLOB_COUNT} value={(blobStore.blobCount ?? 0).toLocaleString(pageLanguage())} />
            {blobStore.path && <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.LOCATION} value={blobStore.path} fullWidth />}
          </Grid>
        ) : (
          <Flex align="center" gap="2" p="3" style={{ backgroundColor: 'var(--gray-2)', borderRadius: '4px' }}>
            <Info size={16} color="var(--gray-8)" />
            <Text color="gray" size="2">{RepositoryProfileStrings.REPOSITORY_TAB.BLOB_STORE_DETAILS_NOT_AVAILABLE}</Text>
          </Flex>
        )}
      </ProfileSection>

      {cleanupPolicies.length > 0 && (
        <ProfileSection
          title={RepositoryProfileStrings.REPOSITORY_TAB.CLEANUP_POLICIES}
          icon={Trash2}
          editPath="admin/repository/cleanup-policies"
        >
          <Grid columns="2" gap="4">
            {cleanupPolicies.map((policy) => (
              <Card key={policy.name} variant="ghost" style={{ backgroundColor: 'var(--gray-2)' }}>
                <Flex direction="column" gap="1">
                  <Text size="2" weight="bold">{policy.name}</Text>
                  <Flex align="center" gap="2">
                    <Clock size={12} color="var(--gray-8)" />
                    <Text size="1" color="gray">
                      {policy.lastRun ? RepositoryProfileStrings.REPOSITORY_TAB.LAST_RUN(new Date(policy.lastRun).toLocaleDateString(pageLanguage())) : RepositoryProfileStrings.REPOSITORY_TAB.NEVER_RUN}
                    </Text>
                  </Flex>
                </Flex>
              </Card>
            ))}
          </Grid>
        </ProfileSection>
      )}

      {routingRule && (
        <ProfileSection
          title={RepositoryProfileStrings.REPOSITORY_TAB.ROUTING_RULES}
          icon={Route}
          editPath="admin/repository/routing-rules"
        >
          <Grid columns="2" gap="4">
            <ConfigRow label={CommonStrings.LABELS.NAME} value={routingRule.name} />
            <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.MODE} value={routingRule.mode} />
            <ConfigRow label={RepositoryProfileStrings.REPOSITORY_TAB.MATCHERS} value={undefined} isList listItems={routingRule.matchers} fullWidth />
          </Grid>
        </ProfileSection>
      )}
    </Box>
  );
}

export default RepositoryTab;
