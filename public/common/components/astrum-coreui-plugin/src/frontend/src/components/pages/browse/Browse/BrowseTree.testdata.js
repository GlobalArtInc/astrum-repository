
export const COMPONENTS = [
  {
    id: "folder1",
    text: "folder1",
    type: "folder",
    leaf: false,
    componentId: null,
    assetId: null,
    packageUrl: null
  },
  {
    id: "component1",
    text: "component1",
    type: "component",
    leaf: false,
    component: null,
    assetId: null,
    packageUrl: null
  },
  {
    id: "asset1",
    text: "asset1",
    type: "asset",
    leaf: true,
    componentId: null,
    assetId: null,
    packageUrl: null
  }
]

export const FOLDER1_CHILDREN = [
  {
    id: "folder1/component1",
    text: "component1",
    type: "component",
    leaf: false,
    componentId: null,
    assetId: null,
    packageUrl: null,
  },
  {
    id: "folder1/component2",
    text: "component2",
    type: "component",
    leaf: false,
    componentId: null,
    assetId: null,
    packageUrl: null,
  }
]

export const COMPONENT1_CHILDREN = [
  {
    id: "folder1/component1/asset1.txt",
    text: "asset1.txt",
    type: "asset",
    leaf: true,
    componentId: null,
    assetId: "asset1",
    packageUrl: null,
  },
  {
    id: "folder1/component1/asset1.txt.md5",
    text: "asset1.txt.md5",
    type: "asset",
    leaf: true,
    componentId: null,
    assetId: "asset1.txt.md5",
    packageUrl: null,
  },
  {
    id: "folder1/component1/asset1.txt.sha1",
    text: "asset1.txt.sha1",
    type: "asset",
    leaf: true,
    componentId: null,
    assetId: "asset1.txt.sha1",
    packageUrl: null,
  },
  {
    id: "folder1/component1/asset1.txt.sha256",
    text: "asset1.txt.sha256",
    type: "asset",
    leaf: true,
    componentId: null,
    assetId: "asset1.txt.sha256",
    packageUrl: null,
  },
  {
    id: "folder1/component1/asset1.txt.sha512",
    text: "asset1.txt.sha512",
    type: "asset",
    leaf: true,
    componentId: null,
    assetId: "asset1.txt.sha512",
    packageUrl: null,
  }
]
