import { auditArea } from '../areas/audit';
import { dashboardArea } from '../areas/dashboard';
import { searchArea } from '../areas/search';
import { settingsArea } from '../areas/settings';
import { uploadArea } from '../areas/upload';
import type { Area } from './area';

export const areas: Area[] = [dashboardArea, searchArea, uploadArea, auditArea, settingsArea];
