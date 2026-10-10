

import React, {useState, useEffect} from 'react';
import {DropdownMenu, Flex, IconButton, Text} from '@radix-ui/themes';
import {Tooltip, usePortalContainer} from '@globalart/astrum-ui-plugin';
import {HelpCircle, ExternalLink} from 'lucide-react';
import {ExtJS} from '@globalart/astrum-ui-plugin';

const DocumentationUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-docs'
};
const ReleaseNotesUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-release-notes'
};
const KnowledgeBaseUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-knowledge'
};
const GlobalArtGuidesUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-guides'
};
const CommunityUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-community'
};
const IssueTrackerUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-issuetracker'
};

const baseUrl = 'https://links.globalart.dev/products/astrum';

function buildUrl(path, utmParams) {
  const params = new URLSearchParams(utmParams);
  return `${baseUrl}/${path}?${params.toString()}`;
}

export default function HelpMenuRadix() {
  const portalContainer = usePortalContainer();
  // Use regular useState instead of ExtJS.useState to avoid hooks issues during logout
  const [version, setVersion] = useState('');

  useEffect(() => {
    try {
      const state = ExtJS.state();
      if (state) {
        setVersion(state.getValue('status')?.version || '');
      }
    } catch (e) {
      setVersion('');
    }
  }, []);

  const getVersionMajorMinor = () => {
    const match = version.match(/^(\d+\.\d+)/);
    return match ? match[1] : '';
  };

  const documentationUrl = buildUrl('docs/' + getVersionMajorMinor(), DocumentationUTMparams);
  const releaseNotesUrl = 'https://links.globalart.dev/products/astrum/release-notes?' + new URLSearchParams(ReleaseNotesUTMparams).toString();
  const knowledgeBaseUrl = buildUrl('kb', KnowledgeBaseUTMparams);
  const guidesUrl = 'https://links.globalart.dev/products/astrum/guides?' + new URLSearchParams(GlobalArtGuidesUTMparams).toString();
  const communityUrl = buildUrl('community', CommunityUTMparams);
  const issueTrackerUrl = buildUrl('issues', IssueTrackerUTMparams);

  return (
    <DropdownMenu.Root>
      <Tooltip content="Help & Documentation">
        <DropdownMenu.Trigger>
          <IconButton variant="outline" size="2" color="gray" aria-label="Help & Documentation" data-analytics-id="astrum-header-help">
            <HelpCircle size={16} />
          </IconButton>
        </DropdownMenu.Trigger>
      </Tooltip>

      <DropdownMenu.Content align="end" container={portalContainer} color="gray" variant="soft">
        <DropdownMenu.Item asChild>
          <a href={documentationUrl} target="_blank" rel="noopener noreferrer">
            <Flex align="center" justify="start" gap="3">
              <Text size="2">Documentation</Text>
              <ExternalLink size={16} />
            </Flex>
          </a>
        </DropdownMenu.Item>

        <DropdownMenu.Item asChild>
          <a href={releaseNotesUrl} target="_blank" rel="noopener noreferrer">
            <Flex align="center" justify="start" gap="3">
              <Text size="2">Release Notes</Text>
              <ExternalLink size={16} />
            </Flex>
          </a>
        </DropdownMenu.Item>

        <DropdownMenu.Item asChild>
          <a href={knowledgeBaseUrl} target="_blank" rel="noopener noreferrer">
            <Flex align="center" justify="start" gap="3">
              <Text size="2">Knowledge Base</Text>
              <ExternalLink size={16} />
            </Flex>
          </a>
        </DropdownMenu.Item>

        <DropdownMenu.Item asChild>
          <a href={guidesUrl} target="_blank" rel="noopener noreferrer">
            <Flex align="center" justify="start" gap="3">
              <Text size="2">GlobalArt Guides</Text>
              <ExternalLink size={16} />
            </Flex>
          </a>
        </DropdownMenu.Item>

        <DropdownMenu.Separator />

        <DropdownMenu.Item asChild>
          <a href={communityUrl} target="_blank" rel="noopener noreferrer">
            <Flex align="center" justify="start" gap="3">
              <Text size="2">Community</Text>
              <ExternalLink size={16} />
            </Flex>
          </a>
        </DropdownMenu.Item>

        <DropdownMenu.Item asChild>
          <a href={issueTrackerUrl} target="_blank" rel="noopener noreferrer">
            <Flex align="center" justify="start" gap="3">
              <Text size="2">Issue Tracker</Text>
              <ExternalLink size={16} />
            </Flex>
          </a>
        </DropdownMenu.Item>

        <DropdownMenu.Separator />

        <DropdownMenu.Label>
          <Text size="1" color="gray">
            Version: {version}
          </Text>
        </DropdownMenu.Label>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}





