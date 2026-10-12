import { auditArea } from '../areas/audit';
import { blobstoresArea } from '../areas/blobstores';
import { browseArea } from '../areas/browse';
import { dashboardArea } from '../areas/dashboard';
import { repositoriesArea } from '../areas/repositories';
import { searchArea } from '../areas/search';
import { selectorsArea } from '../areas/selectors';
import { settingsArea } from '../areas/settings';
import { uploadArea } from '../areas/upload';
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
  selectorsArea,
];
