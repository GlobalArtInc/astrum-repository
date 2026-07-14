

import {ExtJS} from '@globalart/astrum-ui-plugin';

const DB_UPGRADE_DEFAULT = {currentState: null, message: null};

export const UpgradeAlertFunctions = {
  currentState: () => ExtJS.state().getValue('dbUpgrade', DB_UPGRADE_DEFAULT).currentState,
  message: () => ExtJS.state().getValue('dbUpgrade', DB_UPGRADE_DEFAULT).message,
  hasUser: () => !Ext.isEmpty(NX.State.getUser()),
  checkPermissions: () => ExtJS.checkPermission('astrum:atlas:update')
}
