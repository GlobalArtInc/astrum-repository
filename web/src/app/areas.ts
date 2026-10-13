import { auditArea } from '../areas/audit';
import { blobstoresArea } from '../areas/blobstores';
import { browseArea } from '../areas/browse';
import { cleanupArea } from '../areas/cleanup';
import { dashboardArea } from '../areas/dashboard';
import { identityArea } from '../areas/identity';
import { onboardingArea } from '../areas/onboarding';
import { outboundArea } from '../areas/outbound';
import { repositoriesArea } from '../areas/repositories';
import { routingArea } from '../areas/routing';
import { searchArea } from '../areas/search';
import { securityArea } from '../areas/security';
import { selectorsArea } from '../areas/selectors';
import { settingsArea } from '../areas/settings';
import { tasksArea } from '../areas/tasks';
import { uploadArea } from '../areas/upload';
import { userArea } from '../areas/user';
import type { Area } from './area';

export const areas: Area[] = [
  dashboardArea,
  browseArea,
  searchArea,
  uploadArea,
  auditArea,
  settingsArea,
  repositoriesArea,
  blobstoresArea,
  cleanupArea,
  routingArea,
  selectorsArea,
  securityArea,
  identityArea,
  tasksArea,
  outboundArea,
  userArea,
  onboardingArea,
];
