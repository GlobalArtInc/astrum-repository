import { dashboardArea } from '../areas/dashboard';
import { searchArea } from '../areas/search';
import { settingsArea } from '../areas/settings';
import type { Area } from './area';

export const areas: Area[] = [dashboardArea, searchArea, settingsArea];
