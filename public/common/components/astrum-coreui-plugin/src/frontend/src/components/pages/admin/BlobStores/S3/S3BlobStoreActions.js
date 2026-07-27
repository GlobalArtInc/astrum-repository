
import {ValidationUtils} from '@globalart/astrum-ui-plugin';

export default {
  init: () => ({
    bucketConfiguration: {
      bucket: {
        region: 'DEFAULT',
        prefix: ''
      },
      bucketSecurity: null,
      encryption: null,
      advancedBucketConnection: {
        endpoint: '',
        forcePathStyle: false
      },
      failoverBuckets: [],
      activeRegion: null,
    }
  }),

  validation: (data) => {
    const {bucket, bucketSecurity, advancedBucketConnection, failoverBuckets} = data.bucketConfiguration;
    const validationErrors = {
      bucketConfiguration: {
        bucket: {
          name: ValidationUtils.validateNotBlank(bucket?.name)
        },
        bucketSecurity: {},
        advancedBucketConnection: {
          endpoint: ValidationUtils.notBlank(advancedBucketConnection?.endpoint) ?
              ValidationUtils.validateIsUri(advancedBucketConnection?.endpoint) :
              null,
          maxConnectionPoolSize: ValidationUtils.notBlank(advancedBucketConnection?.maxConnectionPoolSize) ?
              ValidationUtils.isInRange({
                value: advancedBucketConnection.maxConnectionPoolSize,
                min: 1,
                max: 1000000000
              }) :
              null
        },
        failoverBuckets: []
      }
    };

    if (ValidationUtils.notBlank(bucketSecurity?.secretAccessKey) ||
        ValidationUtils.notBlank(bucketSecurity?.role) ||
        ValidationUtils.notBlank(bucketSecurity?.sessionToken)) {
      validationErrors.bucketConfiguration.bucketSecurity.accessKeyId =
          ValidationUtils.validateNotBlank(bucketSecurity?.accessKeyId);
    }

    if (ValidationUtils.notBlank(bucketSecurity?.accessKeyId) ||
        ValidationUtils.notBlank(bucketSecurity?.role) ||
        ValidationUtils.notBlank(bucketSecurity?.sessionToken)) {
      validationErrors.bucketConfiguration.bucketSecurity.secretAccessKey =
          ValidationUtils.validateNotBlank(bucketSecurity?.secretAccessKey);
    }

    validationErrors.bucketConfiguration.failoverBuckets = failoverBuckets?.map(failoverBucket => {
      return {
        region: ValidationUtils.validateNotBlank(failoverBucket.region) ||
            (failoverBucket.region === bucket.region ? 'Replication region is the same as primary region' : null) ||
            (failoverBucket.region === 'DEFAULT' ? 'Replication region cannot be DEFAULT' : null),
        bucketName: ValidationUtils.validateNotBlank(failoverBucket.bucketName)
      };
    });

    return validationErrors;
  }
};
