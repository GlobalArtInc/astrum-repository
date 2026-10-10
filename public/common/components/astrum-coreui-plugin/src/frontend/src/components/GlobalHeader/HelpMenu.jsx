

import React from "react";
import {
  AsH4,
  AsStatefulNavigationDropdown,
  AsNavigationDropdown,
  useToggle,
  AsTextLink,
  AsDropdown, AsP
} from '@globalart/astrum-react-shared-components';
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
      <AsStatefulNavigationDropdown
          isOpen={isOpen}
          onToggleCollapse={onToggleCollapse}
          icon={faQuestionCircle}
          title="Help"
          className="as-global-header-help-menu"
          data-analytics-id="as-global-header-help-icon"
      >
        <AsNavigationDropdown.MenuHeader>
          <AsH4>
            Astrum Repository Manager
          </AsH4>
          <AsP>{version}</AsP>
        </AsNavigationDropdown.MenuHeader>

        <button
            onClick={ExtJS.showAbout}
            className="as-dropdown-link"
            data-analytics-id="as-global-header-help-about">
          About
        </button>

        <AsTextLink
            external
            href={`https://links.globalart.dev/products/astrum/docs/${version}?${new URLSearchParams(DocumentationUTMparams).toString()}`}
            className="as-dropdown-link"
            data-analytics-id="as-global-header-help-documentation"
        >
          Documentation
        </AsTextLink>

        <AsTextLink
            external
            href={`https://links.globalart.dev/products/astrum/kb?${new URLSearchParams(KnowledgeBaseUTMparams).toString()}`}
            className="as-dropdown-link"
            data-analytics-id="as-global-header-help-knowledge-base"
        >
          Knowledge Base
        </AsTextLink>

        <AsTextLink
            external
            href={`https://links.globalart.dev/products/astrum/guides?${new URLSearchParams(GlobalArtGuidesUTMparams).toString()}`}
            className="as-dropdown-link"
            data-analytics-id="as-global-header-help-guides"
        >
          GlobalArt Guides
        </AsTextLink>

        <AsTextLink
            external
            href={`https://links.globalart.dev/products/astrum/community?${new URLSearchParams(CommunityUTMparams).toString()}`}
            className="as-dropdown-link"
            data-analytics-id="as-global-header-help-community"
        >
          Community
        </AsTextLink>

        <AsTextLink
            external
            href={`https://links.globalart.dev/products/astrum/issues?${new URLSearchParams(IssueTrackerUTMparams).toString()}`}
            className="as-dropdown-link"
            data-analytics-id="as-global-header-help-issue-tracker"
        >
          Issue Tracker
        </AsTextLink>

        <AsDropdown.Divider />

        <AsTextLink
            external
            href={`https://links.globalart.dev/products/astrum/support?${new URLSearchParams(SupportUTMparams).toString()}`}
            className="as-dropdown-link"
            data-analytics-id="as-global-header-help-support"
        >
          Support
        </AsTextLink>
      </AsStatefulNavigationDropdown>);
}
