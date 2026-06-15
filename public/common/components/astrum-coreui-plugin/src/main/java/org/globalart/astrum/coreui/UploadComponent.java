
package org.globalart.astrum.coreui;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.coreui.internal.UploadService;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.repository.upload.UploadDefinition;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;

import java.util.Collection;
import java.util.stream.Collectors;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Upload {@link DirectComponent}.
 */
@Component
@DirectAction(action = "coreui_Upload")
public class UploadComponent
    extends DirectComponentSupport
{
  private final UploadService uploadService;

  @Autowired
  public UploadComponent(final UploadService uploadService) {
    this.uploadService = checkNotNull(uploadService);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public Collection<UploadDefinition> getUploadDefinitions() {
    return uploadService.getAvailableDefinitions()
        .stream()
        .filter(UploadDefinition::isUiUpload)
        .collect(Collectors.toList()); // NOSONAR
  }
}
