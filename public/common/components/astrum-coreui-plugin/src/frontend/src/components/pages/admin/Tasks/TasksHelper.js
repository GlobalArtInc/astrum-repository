
import {ExtJS, Permissions, APIConstants} from '@globalart/astrum-ui-plugin';
import UIStrings from '../../../../constants/UIStrings';

const {REST: {PUBLIC: {TASKS: tasksUrl}}} = APIConstants;
const {TASKS: {FORM: LABELS }} = UIStrings;

export const canCreateTask = () => ExtJS.checkPermission(Permissions.TASKS.CREATE);
export const canDeleteTask = () => ExtJS.checkPermission(Permissions.TASKS.DELETE);
export const canUpdateTask = () => ExtJS.checkPermission(Permissions.TASKS.UPDATE);
export const canRunTask = () => ExtJS.checkPermission(Permissions.TASKS.START);
export const canStopTask = () => ExtJS.checkPermission(Permissions.TASKS.STOP);

const runTaskUrl = id => `${tasksUrl}/${encodeURIComponent(id)}/run`;
const stopTaskUrl = id => `${tasksUrl}/${encodeURIComponent(id)}/stop`;

export const URLs = {runTaskUrl, stopTaskUrl};

export const NOTIFICATION_CONDITIONS = {
  FAILURE: {
    LABEL: LABELS.SEND_NOTIFICATION_ON.OPTIONS.FAILURE,
    ID: 'FAILURE'
  },
  SUCCESS_FAILURE: {
    ID: 'SUCCESS_FAILURE',
    LABEL: LABELS.SEND_NOTIFICATION_ON.OPTIONS.SUCCESS_FAILURE,
  },
};

export const INITIAL_DATA = {
  typeId: '',
  name: '',
  enabled: true,
  alertEmail: '',
  notificationCondition: NOTIFICATION_CONDITIONS.FAILURE.ID,
  properties: {},
  // TODO Remove when ASTRUM-37051 is done.
  schedule: 'manual',
};
