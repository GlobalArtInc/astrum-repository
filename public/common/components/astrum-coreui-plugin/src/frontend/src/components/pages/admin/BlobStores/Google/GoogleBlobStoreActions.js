
import {ValidationUtils} from '@globalart/astrum-ui-plugin';

export default {
  init: () => ({
    bucketConfiguration: {
      bucketSecurity: {
        authenticationMethod: 'applicationDefault'
      },
      encryption: {
        encryptionType: 'default'
      }
    }
  }),

  validation: (data, pristineData) => {
    const validationErrors = {
      bucketConfiguration: {
        bucket: {
          name: ValidationUtils.validateNotBlank(data.bucketConfiguration?.bucket?.name)
        }
      }
    };

    const isEdit = Boolean(pristineData?.name);

    if (data.bucketConfiguration?.bucketSecurity?.authenticationMethod === 'accountKey' && !isEdit) {
      validationErrors.bucketConfiguration.bucketSecurity = {
        file: ValidationUtils.validateNotBlank(data.bucketConfiguration?.bucketSecurity.file)
      }
    }

    if (data.bucketConfiguration?.encryption?.encryptionType === 'kmsManagedEncryption') {
      validationErrors.bucketConfiguration.encryption = {
        encryptionKey: ValidationUtils.validateNotBlank(data.bucketConfiguration?.encryption.encryptionKey)
      }
    }

    return validationErrors;
  }
};
