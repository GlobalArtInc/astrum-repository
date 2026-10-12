import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';


/**
 * Descriptions for all repository recipes (format + type combinations).
 * Used by RepositoryTypeSelector to display helpful context about each repository type.
 */
export const RECIPE_DESCRIPTIONS: Record<string, string> = {
  // Maven
  'maven2-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_MAVEN_ARTIFACTS_PRIVATE,
  'maven2-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_MAVEN_ARTIFACTS_REMOTE_REPOSITORY,
  'maven2-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_MAVEN_REPOSITORIES_INTO,

  // Docker
  'docker-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_DOCKER_IMAGES_PRIVATE,
  'docker-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_DOCKER_IMAGES_REMOTE_REGISTRY,
  'docker-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_DOCKER_REGISTRIES_INTO,

  // npm
  'npm-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_NPM_PACKAGES_PRIVATE,
  'npm-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_NPM_PACKAGES_REMOTE_REGISTRY,
  'npm-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_NPM_REGISTRIES_INTO,

  // NuGet
  'nuget-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_NUGET_PACKAGES_PRIVATE,
  'nuget-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_NUGET_PACKAGES_REMOTE_REPOSITORY,
  'nuget-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_NUGET_REPOSITORIES_INTO,

  // PyPI
  'pypi-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_PYTHON_PACKAGES_PRIVATE,
  'pypi-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_PYTHON_PACKAGES_REMOTE_REPOSITORY,
  'pypi-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_PYPI_REPOSITORIES_INTO,

  // RubyGems
  'rubygems-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_RUBY_GEMS_PRIVATE,
  'rubygems-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_RUBY_GEMS_REMOTE_REPOSITORY,
  'rubygems-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_RUBYGEMS_REPOSITORIES_INTO,

  // Yum
  'yum-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_RPM_PACKAGES_PRIVATE,
  'yum-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_RPM_PACKAGES_REMOTE_YUM,
  'yum-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_YUM_REPOSITORIES_INTO,

  // APT
  'apt-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_DEBIAN_PACKAGES_PRIVATE,
  'apt-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_DEBIAN_PACKAGES_REMOTE_APT,

  // Alpine
  'alpine-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_ALPINE_LINUX_PACKAGES,
  'alpine-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_ALPINE_LINUX_PACKAGES_REMOTE,
  'alpine-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_ALPINE_REPOSITORIES_INTO,

  // Helm
  'helm-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_HELM_CHARTS_PRIVATE,
  'helm-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_HELM_CHARTS_REMOTE_REPOSITORY,

  // Raw
  'raw-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_ANY_BINARY_FILES_PRIVATE,
  'raw-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_BINARY_FILES_REMOTE_LOCATION,
  'raw-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_RAW_REPOSITORIES_INTO,

  // Go
  'go-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_GO_MODULES_REMOTE_REPOSITORY,
  'go-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_GO_REPOSITORIES_INTO,

  // R
  'r-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_R_PACKAGES_PRIVATE,
  'r-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_R_PACKAGES_REMOTE_REPOSITORY,
  'r-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_R_REPOSITORIES_INTO,

  // Conan
  'conan-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_CONAN_C_C_PACKAGES,

  // Conda
  'conda-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_CONDA_PACKAGES_REMOTE_REPOSITORY,

  // CocoaPods
  'cocoapods-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_COCOAPODS_IOS_MACOS_DEVELOPMENT,

  // Git LFS
  'gitlfs-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_LARGE_FILES_GIT_LFS,

  // P2
  'p2-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_ECLIPSE_P2_ARTIFACTS_METADATA,

  // Terraform
  'terraform-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_TERRAFORM_MODULES_PROVIDERS,

  // Ansible Galaxy
  'ansiblegalaxy-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_ANSIBLE_COLLECTIONS_PRIVATE,
  'ansiblegalaxy-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_ANSIBLE_COLLECTIONS_REMOTE_REPOSITORY,
  'ansiblegalaxy-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_ANSIBLE_GALAXY_REPOSITORIES,

  // Composer
  'composer-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_PHP_PACKAGES_REMOTE_COMPOSER,
  'composer-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_PHP_COMPOSER_PACKAGES,
  'composer-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_COMPOSER_REPOSITORIES_INTO,

  // Swift
  'swift-hosted': RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_SWIFT_PACKAGES_PRIVATE,
  'swift-proxy': RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_SWIFT_PACKAGES_REMOTE_REGISTRY,
  'swift-group': RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_SWIFT_REGISTRIES_INTO,
};

/**
 * General descriptions for repository types.
 */
export const TYPE_DESCRIPTIONS: Record<string, string> = {
  hosted: RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_OWN_COMPONENTS_ARTIFACTS_PRIVATE,
  proxy: RepositoriesPageStrings.TYPE_DESCRIPTIONS.CACHE_COMPONENTS_REMOTE_REGISTRY_LIKE,
  group: RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMBINE_MULTIPLE_REPOSITORIES_HOSTED_PROXY,
};

/**
 * General descriptions for technology formats.
 */
export const FORMAT_DESCRIPTIONS: Record<string, string> = {
  maven2: RepositoriesPageStrings.TYPE_DESCRIPTIONS.STANDARD_JAVA_DEVELOPMENT_SUPPORTS_SNAPSHOTS,
  npm: RepositoriesPageStrings.TYPE_DESCRIPTIONS.PACKAGE_MANAGER_JAVASCRIPT_NODE_JS,
  docker: RepositoriesPageStrings.TYPE_DESCRIPTIONS.CONTAINER_IMAGE_REGISTRY_DOCKER_OCI,
  nuget: RepositoriesPageStrings.TYPE_DESCRIPTIONS.PACKAGE_MANAGER_NET_DEVELOPMENT,
  pypi: RepositoriesPageStrings.TYPE_DESCRIPTIONS.OFFICIAL_THIRD_PARTY_SOFTWARE_REPOSITORY,
  rubygems: RepositoriesPageStrings.TYPE_DESCRIPTIONS.STANDARD_RUBY_PACKAGE_MANAGER,
  helm: RepositoriesPageStrings.TYPE_DESCRIPTIONS.PACKAGE_MANAGER_KUBERNETES_APPLICATIONS,
  go: RepositoriesPageStrings.TYPE_DESCRIPTIONS.PROXY_REGISTRY_GO_MODULES,
  terraform: RepositoriesPageStrings.TYPE_DESCRIPTIONS.REGISTRY_TERRAFORM_PROVIDERS_MODULES,
  cocoapods: RepositoriesPageStrings.TYPE_DESCRIPTIONS.DEPENDENCY_MANAGER_SWIFT_OBJECTIVE_C,
  r: RepositoriesPageStrings.TYPE_DESCRIPTIONS.COMPREHENSIVE_R_ARCHIVE_NETWORK_STATISTICAL,
  conan: RepositoriesPageStrings.TYPE_DESCRIPTIONS.OPEN_SOURCE_PACKAGE_MANAGER_C,
  conda: RepositoriesPageStrings.TYPE_DESCRIPTIONS.PACKAGE_DEPENDENCY_ENVIRONMENT_MANAGEMENT_ANY,
  gitlfs: RepositoriesPageStrings.TYPE_DESCRIPTIONS.GIT_LARGE_FILE_STORAGE_VERSIONING,
  p2: RepositoriesPageStrings.TYPE_DESCRIPTIONS.UPDATE_MECHANISM_ECLIPSE_PLUGINS_APPLICATIONS,
  yum: RepositoriesPageStrings.TYPE_DESCRIPTIONS.PRIMARY_PACKAGE_MANAGEMENT_TOOL_RPM,
  apt: RepositoriesPageStrings.TYPE_DESCRIPTIONS.ADVANCED_PACKAGE_TOOL_DEBIAN_UBUNTU,
  alpine: RepositoriesPageStrings.TYPE_DESCRIPTIONS.PACKAGE_MANAGER_ALPINE_LINUX_DISTRIBUTIONS,
  raw: RepositoriesPageStrings.TYPE_DESCRIPTIONS.STORE_ANY_FILE_FORMAT_WITHOUT,
  ansiblegalaxy: RepositoriesPageStrings.TYPE_DESCRIPTIONS.REPOSITORY_ANSIBLE_COLLECTIONS_CONTAINING_ROLES,
  composer: RepositoriesPageStrings.TYPE_DESCRIPTIONS.DEPENDENCY_MANAGER_PHP,
  swift: RepositoriesPageStrings.TYPE_DESCRIPTIONS.PACKAGE_REGISTRY_SWIFT_PACKAGE_MANAGER,
};

/**
 * Get description for a repository type.
 */
export function getTypeDescription(type: string): string {
  return TYPE_DESCRIPTIONS[type] || RepositoriesPageStrings.TYPE_DESCRIPTIONS.CREATE_REPOSITORY(type);
}

/**
 * Get description for a format.
 */
export function getFormatDescription(format: string): string {
  return FORMAT_DESCRIPTIONS[format] || RepositoriesPageStrings.TYPE_DESCRIPTIONS.MANAGE_COMPONENTS(format);
}

/**
 * Get description for a repository recipe.
 */
export function getRecipeDescription(format: string, type: string): string {
  const key = `${format}-${type}`;
  return RECIPE_DESCRIPTIONS[key] || RepositoriesPageStrings.TYPE_DESCRIPTIONS.CREATE_REPOSITORY_2(format, type);
}
