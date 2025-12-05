
package org.globalart.astrum.internal.atlas.customizers;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.atlas.SystemInformationGenerator;
import org.globalart.astrum.supportzip.GeneratedContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.REQUIRED;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.SYSINFO;
import org.springframework.stereotype.Component;

/**
 * Adds system information report to support bundle.
 *
 * @since 2.7
 */
@Component
public class SystemInformationCustomizer
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final SystemInformationGenerator systemInformationGenerator;

  private final ObjectMapper objectMapper;

  @Autowired
  public SystemInformationCustomizer(final SystemInformationGenerator systemInformationGenerator) {
    this.systemInformationGenerator = checkNotNull(systemInformationGenerator);
    this.objectMapper = new ObjectMapper();
  }

  @Override
  public void customize(final SupportBundle supportBundle) {
    supportBundle.add(new GeneratedContentSourceSupport(SYSINFO, "info/sysinfo.json", REQUIRED)
    {
      @Override
      protected void generate(final File file) {
        Map<String, Object> report = systemInformationGenerator.report();
        try (FileOutputStream fos = new FileOutputStream(file)) {
          objectMapper.writerWithDefaultPrettyPrinter().writeValue(fos, report);
        }
        catch (IOException e) {
          throw new UncheckedIOException(e);
        }
      }
    });
  }
}
