

import {
  FileSearch,
  Globe,
  Users,
  Cloud,
  UserCheck,
  HeartPulse,
  Clock,
  Link,
  Server,
  Megaphone,
  Scissors,
  KeyRound,
  Calendar,
  HardDrive,
  Palette,
  Settings,
  ArrowUpCircle,
  Webhook,
  GitBranch,
  Puzzle,
} from 'lucide-react';

// `firewall.audit` was removed from this map: post-migration the new UI never renders
// capabilities of that type (filtered at useCapabilitiesApi.fetchCapabilities() and not
// advertised by /v1/capabilities/types). Firewall configuration belongs on the repository
// config page now. The legacy REST API surface remains for hardcoded client compatibility.
// The previously-imported `Shield` icon has been pruned along with the lookup entry.

export const TYPE_ICONS: Record<string, React.ComponentType<{ size?: number; className?: string }>> = {
  audit: FileSearch,
  baseurl: Globe,
  crowd: Users,
  customs3regions: Cloud,
  defaultrole: UserCheck,
  healthcheck: HeartPulse,
  'license-expiration': Clock,
  LegacyUrlCapability: Link,
  'node.identity': Server,
  outreach: Megaphone,
  OutreachManagementCapability: Megaphone,
  'browse.trim': Scissors,
  rutauth: KeyRound,
  'scheduling.scheduler': Calendar,
  StorageSettings: HardDrive,
  'rapture.branding': Palette,
  'rapture.settings': Settings,
  migration: ArrowUpCircle,
  'webhook.global': Webhook,
  'webhook.repository': GitBranch,
};

export const DEFAULT_TYPE_ICON = Puzzle;
