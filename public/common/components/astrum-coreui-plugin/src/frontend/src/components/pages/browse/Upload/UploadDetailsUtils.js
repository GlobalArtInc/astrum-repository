

/**
 * In compound field structures, this value is used to denote the field which is mapped to the parent path in the
 * form submission data.  For instance the 'bar' data in { foo: { _: 'bar', a: 'baz' } } would be submitted
 * under the key 'foo' while the 'baz' data would be submitted under the key 'foo.a'
 */
export const COMPOUND_FIELD_PARENT_NAME = '_';

/**
 * A regex which can be used to check whether a given field key represents an asset upload name and if so to capture
 * the numeric id of the asset in its first (and only) capture group
 */
export const ASSET_NUM_MATCHER = /^asset(\d+)$/;

export const MAVEN_FORMAT = 'maven2';
export const MAVEN_COMPONENT_COORDS_GROUP = 'Component coordinates';
export const MAVEN_GENERATE_POM_FIELD_NAME = 'generate-pom';
export const MAVEN_PACKAGING_FIELD_NAME = 'packaging';

export const TERRAFORM_FORMAT = 'terraform';
export const TERRAFORM_MODULE_COORDS_GROUP = 'Module Coordinates';
export const TERRAFORM_PROVIDER_COORDS_GROUP = 'Provider Coordinates';
export const TERRAFORM_MODULE_COORDS_GROUP_HELP = 'Applicable only when the Upload Type is Module';
export const TERRAFORM_PROVIDER_COORDS_GROUP_HELP = 'Applicable only when the Upload Type is Provider';
export const TERRAFORM_UPLOAD_TYPE_FIELD_NAME = 'uploadType';
export const TERRAFORM_UPLOAD_TYPE_MODULE = 'module';
export const TERRAFORM_UPLOAD_TYPE_PROVIDER = 'provider';
