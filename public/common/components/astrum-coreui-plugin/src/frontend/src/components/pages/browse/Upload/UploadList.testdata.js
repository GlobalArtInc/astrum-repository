
export const REPOS = [
  {
    "name": "maven-public",
    "url": "http://localhost:8081/repository/maven-public",
    "online": true,
    "storage": {
      "blobStoreName": "default",
      "strictContentTypeValidation": false,
      "writePolicy": "ALLOW_ONCE"
    },
    "cleanup": null,
    "maven": {
      "versionPolicy": "RELEASE",
      "layoutPolicy": "STRICT",
      "contentDisposition": "INLINE"
    },
    "component": {"proprietaryComponents": false},
    "format": "raw",
    "type": "hosted"
  },
  {
    "name": "maven-releases",
    "url": "http://localhost:8081/repository/maven-releases",
    "online": true,
    "storage": {
      "blobStoreName": "default",
      "strictContentTypeValidation": false,
      "writePolicy": "ALLOW_ONCE"
    },
    "cleanup": null,
    "maven": {
      "versionPolicy": "RELEASE",
      "layoutPolicy": "STRICT",
      "contentDisposition": "INLINE"
    },
    "component": {"proprietaryComponents": false},
    "format": "maven2",
    "type": "hosted"
  },
  {
    "name": "nuget-hosted",
    "format": "nuget",
    "url": "http://localhost:8081/repository/nuget-hosted",
    "online": true,
    "storage": {
      "blobStoreName": "default",
      "strictContentTypeValidation": true,
      "writePolicy": "ALLOW"
    },
    "cleanup": null,
    "component": {"proprietaryComponents": false},
    "type": "hosted"
  },
];

export const FORMATS = [
  {uiUpload: true, format: 'helm'},
  {uiUpload: true, format: 'r'},
  {uiUpload: true, format: 'pypi'},
  {uiUpload: true, format: 'yum'},
  {uiUpload: true, format: 'rubygems'},
  {uiUpload: true, format: 'npm'},
  {uiUpload: true, format: 'raw'},
  {uiUpload: true, format: 'apt'},
  {uiUpload: true, format: 'nuget'},
  {uiUpload: true, format: 'maven2'},
];

export const FIELDS = {
  NAME: 'name',
  FORMAT: 'format',
};
