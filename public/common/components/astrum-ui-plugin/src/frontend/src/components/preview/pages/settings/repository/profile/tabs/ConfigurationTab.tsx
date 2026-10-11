

import React from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import {
  Settings,
  HardDrive,
  Server,
  Globe,
  Users,
  Trash2,
  Copy,
} from 'lucide-react';
import type { RepositoryProfileData } from '../types';
import { ExtJS } from '@globalart/astrum-ui-plugin';
import RepositoryProfileStrings from '../../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import CommonStrings from '../../../../../constants/CommonStrings';
import { formatDuration } from '../../../../../shared/utils/formatLocale';

// =============================================================================
// Types
// =============================================================================

interface ConfigurationTabProps {
  repository: RepositoryProfileData;
}

// =============================================================================
// Helper Components
// =============================================================================

interface ConfigSectionProps {
  title: string;
  icon: React.ElementType;
  children: React.ReactNode;
}

function ConfigSection({ title, icon: Icon, children }: ConfigSectionProps): JSX.Element {
  return (
    <Box className="profile-section__card" mb="4">
      <Flex align="center" gap="2" mb="4">
        <Icon size={18} />
        <Text weight="bold">{title}</Text>
      </Flex>
      {children}
    </Box>
  );
}

interface ConfigRowProps {
  label: string;
  value: string | number | boolean | undefined | null;
  isCode?: boolean;
  isList?: boolean;
  listItems?: string[];
}

function ConfigRow({ label, value, isCode, isList, listItems }: ConfigRowProps): JSX.Element | null {
  // Don't render if value is undefined/null and not a list
  if (!isList && (value === undefined || value === null)) return null;

  let displayValue: React.ReactNode;

  if (isList && listItems) {
    displayValue = listItems.length > 0 ? (
      <Flex direction="column" gap="1">
        {listItems.map((item, i) => (
          <Text key={i} className={isCode ? 'profile-section__value--code' : undefined}>
            {item}
          </Text>
        ))}
      </Flex>
    ) : '—';
  } else if (typeof value === 'boolean') {
    displayValue = value ? RepositoryProfileStrings.CONFIGURATION.YES : RepositoryProfileStrings.CONFIGURATION.NO;
  } else if (isCode) {
    displayValue = <code className="profile-section__value--code">{String(value)}</code>;
  } else {
    displayValue = String(value);
  }

  return (
    <Box className="profile-section__row">
      <Text className="profile-section__label">{label}</Text>
      <Text className="profile-section__value">{displayValue}</Text>
    </Box>
  );
}

// =============================================================================
// Component
// =============================================================================

/**
 * ConfigurationTab - Displays all repository settings in read-only format
 *
 * Organized by section:
 * - General: Name, Format, Type, Online status
 * - Storage: Blob Store, Strict Content Validation, Write Policy
 * - Proxy (if applicable): Remote URL, HTTP settings, Caching
 * - Hosted (if applicable): Deployment Policy
 * - Group (if applicable): Member repositories
 * - Cleanup: Associated cleanup policies
 * - Format-specific settings
 */
export function ConfigurationTab({ repository }: ConfigurationTabProps): JSX.Element {
  const attrs = repository.attributes || {};
  const storage = attrs.storage;
  const proxy = attrs.proxy;
  const negativeCache = attrs.negativeCache;
  const httpClient = attrs.httpClient;
  const group = attrs.group;
  const cleanup = attrs.cleanup;
  const component = attrs.component;
  const maven = attrs.maven;
  const docker = attrs.docker;
  const npm = attrs.npm;
  const nugetProxy = attrs.nugetProxy;
  const apt = attrs.apt;
  const yum = attrs.yum;
  const raw = attrs.raw;
  const replication = attrs.replication;

  const isProxy = repository.type === 'proxy';
  const isHosted = repository.type === 'hosted';
  const isGroup = repository.type === 'group';

  const isCloud = ExtJS.useState?.(() => ExtJS.state()?.getValue?.('isCloud'));

  return (
    <Box>
      {/* General Settings */}
      <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.GENERAL} icon={Settings}>
        <ConfigRow label={CommonStrings.LABELS.NAME} value={repository.name} />
        <ConfigRow label={CommonStrings.LABELS.FORMAT} value={repository.format} />
        <ConfigRow label={CommonStrings.LABELS.TYPE} value={repository.type} />
        <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.RECIPE} value={repository.recipe} />
        <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.ONLINE} value={repository.online} />
        <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.URL} value={repository.url} isCode />
        {repository.routingRuleId && (
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.ROUTING_RULE} value={repository.routingRuleId} />
        )}
      </ConfigSection>

      {/* Storage Settings */}
      <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.STORAGE} icon={HardDrive}>
        <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.BLOB_STORE} value={storage?.blobStoreName} />
        <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.STRICT_CONTENT_TYPE_VALIDATION} value={storage?.strictContentTypeValidation} />
        {isHosted && <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.WRITE_POLICY} value={storage?.writePolicy} />}
      </ConfigSection>

      {/* Proxy Settings */}
      {isProxy && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.PROXY} icon={Globe}>
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.REMOTE_STORAGE} value={proxy?.remoteUrl} isCode />
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.CONTENT_MAX_AGE} value={proxy?.contentMaxAge !== undefined ? formatDuration(proxy.contentMaxAge, 'minute') : undefined} />
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.METADATA_MAX_AGE} value={proxy?.metadataMaxAge !== undefined ? formatDuration(proxy.metadataMaxAge, 'minute') : undefined} />

          {negativeCache && (
            <>
              <Text size="2" weight="medium" mt="3" mb="2" style={{ color: 'var(--gray-11)' }}>
                {RepositoryProfileStrings.CONFIGURATION.NEGATIVE_CACHE}
              </Text>
              <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.ENABLED} value={negativeCache.enabled} />
              <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.TIME_LIVE} value={negativeCache.timeToLive !== undefined ? formatDuration(negativeCache.timeToLive, 'minute') : undefined} />
            </>
          )}

          {httpClient && (
            <>
              <Text size="2" weight="medium" mt="3" mb="2" style={{ color: 'var(--gray-11)' }}>
                {RepositoryProfileStrings.CONFIGURATION.HTTP_CONNECTION}
              </Text>
              <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.BLOCKED} value={httpClient.blocked} />
              <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.AUTO_BLOCK} value={httpClient.autoBlock} />
              {httpClient.connection && (
                <>
                  <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.RETRIES} value={httpClient.connection.retries} />
                  <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.TIMEOUT} value={httpClient.connection.timeout !== undefined ? formatDuration(httpClient.connection.timeout, 'second') : undefined} />
                  <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.USE_TRUST_STORE} value={httpClient.connection.useTrustStore} />
                  <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.ENABLE_CIRCULAR_REDIRECTS} value={httpClient.connection.enableCircularRedirects} />
                  <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.ENABLE_COOKIES} value={httpClient.connection.enableCookies} />
                </>
              )}
              {httpClient.authentication && (
                <>
                  <Text size="2" weight="medium" mt="2" mb="2" style={{ color: 'var(--gray-11)' }}>
                    {RepositoryProfileStrings.CONFIGURATION.AUTHENTICATION}
                  </Text>
                  <ConfigRow label={CommonStrings.LABELS.TYPE} value={httpClient.authentication.type} />
                  <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.USERNAME} value={httpClient.authentication.username} />
                </>
              )}
            </>
          )}
        </ConfigSection>
      )}

      {/* Hosted Settings */}
      {isHosted && component && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.HOSTED} icon={Server}>
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.PROPRIETARY_COMPONENTS} value={component.proprietaryComponents} />
        </ConfigSection>
      )}

      {/* Group Settings */}
      {isGroup && group && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.GROUP} icon={Users}>
          <ConfigRow
            label={RepositoryProfileStrings.CONFIGURATION.MEMBER_REPOSITORIES}
            value={undefined}
            isList
            listItems={group.memberNames || []}
          />
        </ConfigSection>
      )}

      {/* Cleanup Policies */}
      {cleanup && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.CLEANUP} icon={Trash2}>
          <ConfigRow
            label={RepositoryProfileStrings.CONFIGURATION.CLEANUP_POLICIES}
            value={undefined}
            isList
            listItems={cleanup.policyName || []}
          />
        </ConfigSection>
      )}

      {/* Replication */}
      {replication && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.REPLICATION} icon={Copy}>
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.PREEMPTIVE_PULL_ENABLED} value={replication.preemptivePullEnabled} />
        </ConfigSection>
      )}

      {/* Format-Specific Settings */}
      {maven && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.MAVEN} icon={Settings}>
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.VERSION_POLICY} value={maven.versionPolicy} />
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.LAYOUT_POLICY} value={maven.layoutPolicy} />
          {!isCloud && <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.CONTENT_DISPOSITION} value={maven.contentDisposition} />}
        </ConfigSection>
      )}

      {docker && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.DOCKER} icon={Settings}>
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.HTTP_PORT} value={docker.httpPort} />
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.HTTPS_PORT} value={docker.httpsPort} />
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.V1_ENABLED} value={docker.v1Enabled} />
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.FORCE_BASIC_AUTH} value={docker.forceBasicAuth} />
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.SUBDOMAIN} value={docker.subdomain} />
        </ConfigSection>
      )}

      {npm && (
        <ConfigSection title="npm" icon={Settings}>
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.REMOVE_QUARANTINED} value={npm.removeQuarantined} />
        </ConfigSection>
      )}

      {nugetProxy && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.NUGET} icon={Settings}>
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.QUERY_CACHE_ITEM_MAX_AGE} value={nugetProxy.queryCacheItemMaxAge !== undefined ? formatDuration(nugetProxy.queryCacheItemMaxAge, 'second') : undefined} />
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.NUGET_VERSION} value={nugetProxy.nugetVersion} />
        </ConfigSection>
      )}

      {apt && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.APT} icon={Settings}>
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.DISTRIBUTION} value={apt.distribution} />
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.FLAT} value={apt.flat} />
        </ConfigSection>
      )}

      {yum && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.YUM} icon={Settings}>
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.REPODATA_DEPTH} value={yum.repodataDepth} />
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.DEPLOY_POLICY} value={yum.deployPolicy} />
        </ConfigSection>
      )}

      {raw && !isCloud && (
        <ConfigSection title={RepositoryProfileStrings.CONFIGURATION.RAW} icon={Settings}>
          <ConfigRow label={RepositoryProfileStrings.CONFIGURATION.CONTENT_DISPOSITION} value={raw.contentDisposition} />
        </ConfigSection>
      )}
    </Box>
  );
}

export default ConfigurationTab;


