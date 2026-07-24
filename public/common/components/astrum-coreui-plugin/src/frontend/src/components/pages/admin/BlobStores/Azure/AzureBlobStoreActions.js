
import UIStrings, {ValidationUtils} from '@globalart/astrum-ui-plugin';

export default {
  init: () => ({
    bucketConfiguration: {
      authentication: {
        authenticationMethod: 'MANAGEDIDENTITY'
      }
    }
  }),

  validation: (data) => {
    const validationErrors = {
      bucketConfiguration: {
        accountName: ValidationUtils.validateNotBlank(data.bucketConfiguration?.accountName),
        containerName: ValidationUtils.validateNotBlank(data.bucketConfiguration?.containerName)
      }
    };

    if (data.bucketConfiguration?.authentication?.authenticationMethod === 'ACCOUNTKEY') {
      validationErrors.bucketConfiguration.authentication = {
        accountKey: ValidationUtils.validateNotBlank(data.bucketConfiguration.authentication.accountKey)
      }
    }

    return validationErrors;
  }
};
