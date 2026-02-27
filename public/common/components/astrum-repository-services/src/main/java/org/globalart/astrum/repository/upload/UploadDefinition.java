
package org.globalart.astrum.repository.upload;

import java.util.Collections;
import java.util.List;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Describes the fields associated with a component, fields associated with an asset and whether the format supports
 * multiple asset uploads.
 *
 * @since 3.7
 */
public class UploadDefinition
{

  private final boolean uiUpload;

  private final boolean apiUpload;

  private final boolean multipleUpload;

  private final String format;

  private final List<UploadFieldDefinition> componentFields;

  private final List<UploadFieldDefinition> assetFields;

  private final UploadRegexMap regexMap;

  public UploadDefinition(
      final String format,
      final boolean uiUpload,
      final boolean apiUpload,
      final boolean multipleUpload,
      final List<UploadFieldDefinition> componentFields,
      final List<UploadFieldDefinition> assetFields,
      final UploadRegexMap regexMap)
  {
    this.uiUpload = uiUpload;
    this.apiUpload = apiUpload;
    this.multipleUpload = multipleUpload;
    this.format = checkNotNull(format);
    this.componentFields = Collections.unmodifiableList(checkNotNull(componentFields));
    this.assetFields = Collections.unmodifiableList(checkNotNull(assetFields));
    this.regexMap = regexMap;
  }

  public UploadDefinition(
      final String format,
      final boolean uiUpload,
      final boolean apiUpload,
      final boolean multipleUpload,
      final List<UploadFieldDefinition> componentFields,
      final List<UploadFieldDefinition> assetFields)
  {
    this(format, uiUpload, apiUpload, multipleUpload, componentFields, assetFields, null);
  }

  /**
   * Whether uploads through the UI are allowed by the available handler.
   */
  public boolean isUiUpload() {
    return uiUpload;
  }

  /**
   * Whether uploads through the API are allowed by the available handler.
   */
  public boolean isApiUpload() {
    return apiUpload;
  }

  /**
   * Whether multiple uploads are supported by the available handler.
   */
  public boolean isMultipleUpload() {
    return multipleUpload;
  }

  /**
   * The repository format
   */
  public String getFormat() {
    return format;
  }

  /**
   * The fields associated with the component for uploads of this format.
   */
  public List<UploadFieldDefinition> getComponentFields() {
    return componentFields;
  }

  /**
   * The fields associated with the asset for uploads of this format.
   */
  public List<UploadFieldDefinition> getAssetFields() {
    return assetFields;
  }

  /**
   * The mapper to use for file names.
   * 
   * @since 3.8
   */
  @Nullable
  public UploadRegexMap getRegexMap() {
    return regexMap;
  }
}
