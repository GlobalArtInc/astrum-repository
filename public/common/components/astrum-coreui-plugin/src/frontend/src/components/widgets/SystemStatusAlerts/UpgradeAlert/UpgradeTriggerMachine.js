

import {ExtJS} from "@globalart/astrum-ui-plugin";
import axios from "axios";
import {createMachine} from "xstate";

const UpgradeTriggerMachine = createMachine({
    id: 'UpgradeTriggerMachine',
    initial: 'loaded',
    states: {
      loaded: {
        on: {
          SAVE_MODAL: {
            target: 'triggerUpdate'
          }
        }
      },
      triggerUpdate: {
        invoke: {
          src: 'triggerUpdate',
          onDone: {
            target: 'loaded',
            actions: 'closeModal'
          },
        }
      }
    },
  },
  {
    actions: {
      closeModal: () => ExtJS.state().setValue('dbUpgrade', {currentState: 'astrumUpgradeInProgress'})
    },
    services: {
      triggerUpdate: () => axios.post('service/rest/v1/clustered/upgrade-database-schema')
    }
  });

export default UpgradeTriggerMachine;
