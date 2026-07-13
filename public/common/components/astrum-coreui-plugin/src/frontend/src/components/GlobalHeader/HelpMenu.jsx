

import React from "react";
import {
  NxH4,
  NxStatefulNavigationDropdown,
  NxNavigationDropdown,
  useToggle,
  NxTextLink,
  NxDropdown, NxP
} from '@globalart/react-shared-components';
import { ExtJS } from '@globalart/astrum-ui-plugin';
import { faQuestionCircle } from '@fortawesome/free-solid-svg-icons';

export const DocumentationUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-docs'
};
export const KnowledgeBaseUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-knowledge'
};
export const GlobalArtGuidesUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-guides'
};
export const CommunityUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-community'
};
export const IssueTrackerUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-issuetracker'
};
export const SupportUTMparams = {
  utm_medium: 'product',
  utm_source: 'astrum_repo',
  utm_campaign: 'menu-support'
};

export default function HelpMenu() {
  const [isOpen, onToggleCollapse] = useToggle(false);

  const version = ExtJS.useState(() => ExtJS.state().getVersion());

  return (
      <NxStatefulNavigationDropdown
          isOpen={isOpen}
          onToggleCollapse={onToggleCollapse}
          icon={faQuestionCircle}
          title="Help"
          className="as-global-header-help-menu"
          data-analytics-id="as-global-header-help-icon"
      >
        <NxNavigationDropdown.MenuHeader>
          <NxH4>
            Astrum Repository Manager
          </NxH4>
          <NxP>{version}</NxP>
        </NxNavigationDropdown.MenuHeader>

        <button
            onClick={ExtJS.showAbout}
            className="nx-dropdown-link"
            data-analytics-id="as-global-header-help-about">
          About
        </button>

        <NxTextLink
            external
            href={`https://links.globalart.dev/products/astrum/docs/${version}?${new URLSearchParams(DocumentationUTMparams).toString()}`}
            className="nx-dropdown-link"
            data-analytics-id="as-global-header-help-documentation"
        >
          Documentation
        </NxTextLink>

        <NxTextLink
            external
            href={`https://links.globalart.dev/products/astrum/kb?${new URLSearchParams(KnowledgeBaseUTMparams).toString()}`}
            className="nx-dropdown-link"
            data-analytics-id="as-global-header-help-knowledge-base"
        >
          Knowledge Base
        </NxTextLink>

        <NxTextLink
            external
            href={`https://links.globalart.dev/products/astrum/guides?${new URLSearchParams(GlobalArtGuidesUTMparams).toString()}`}
            className="nx-dropdown-link"
            data-analytics-id="as-global-header-help-guides"
        >
          GlobalArt Guides
        </NxTextLink>

        <NxTextLink
            external
            href={`https://links.globalart.dev/products/astrum/community?${new URLSearchParams(CommunityUTMparams).toString()}`}
            className="nx-dropdown-link"
            data-analytics-id="as-global-header-help-community"
        >
          Community
        </NxTextLink>

        <NxTextLink
            external
            href={`https://links.globalart.dev/products/astrum/issues?${new URLSearchParams(IssueTrackerUTMparams).toString()}`}
            className="nx-dropdown-link"
            data-analytics-id="as-global-header-help-issue-tracker"
        >
          Issue Tracker
        </NxTextLink>

        <NxDropdown.Divider />

        <NxTextLink
            external
            href={`https://links.globalart.dev/products/astrum/support?${new URLSearchParams(SupportUTMparams).toString()}`}
            className="nx-dropdown-link"
            data-analytics-id="as-global-header-help-support"
        >
          Support
        </NxTextLink>
      </NxStatefulNavigationDropdown>);
}
