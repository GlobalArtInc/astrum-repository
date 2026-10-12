import {UIStrings} from '@globalart/astrum-ui-plugin';
import localize from '@globalart/astrum-ui-plugin/src/frontend/src/interface/localize';
import AccountStrings from './pages/user/AccountStrings';
import UpgradeAlertStrings from './UpgradeAlertStrings';
import UpgradeTriggerModalStrings from './UpgradeTriggerModalStrings';
import TelemetryStrings from './pages/telemetry/TelemetryStrings';
import ShellStrings from './ShellStrings';
import PageStrings from './PageStrings';

export default {
  ...UIStrings,
  ...localize('react-coreui', {

    // telemetry
    ...TelemetryStrings,

    // user
    ...AccountStrings,
    ...UpgradeAlertStrings,
    ...UpgradeTriggerModalStrings,

    ...ShellStrings,
    ...PageStrings,
  })
};
