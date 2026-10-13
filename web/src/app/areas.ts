import { apiArea } from '../areas/api';
import { auditArea } from '../areas/audit';
import { blobstoresArea } from '../areas/blobstores';
import { browseArea } from '../areas/browse';
import { cleanupArea } from '../areas/cleanup';
import { dashboardArea } from '../areas/dashboard';
import { identityArea } from '../areas/identity';
import { onboardingArea } from '../areas/onboarding';
import { repositoriesArea } from '../areas/repositories';
import { searchArea } from '../areas/search';
import { securityArea } from '../areas/security';
import { selectorsArea } from '../areas/selectors';
import { settingsArea } from '../areas/settings';
import { supportArea } from '../areas/support';
import { tasksArea } from '../areas/tasks';
import { uploadArea } from '../areas/upload';
import { userArea } from '../areas/user';
import type { Area } from './area';

export const areas: Area[] = [
  dashboardArea,
  browseArea,
  searchArea,
  uploadArea,
  apiArea,
  auditArea,
  settingsArea,
  repositoriesArea,
  blobstoresArea,
  cleanupArea,
  selectorsArea,
  securityArea,
  identityArea,
  tasksArea,
  supportArea,
  userArea,
  onboardingArea,
];
