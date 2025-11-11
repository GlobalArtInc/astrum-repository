
package org.globalart.astrum.internal.datastore;

import java.io.File;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.supportzip.ExportSecurityData;
import org.globalart.astrum.supportzip.GeneratedContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Add {@link SupportBundle} to export {@link Type#SECURITY} data to serialized files.
 *
 * @since 3.29
 */
@Component
public class SecurityDatabase
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String WORK_PATH = "db";

  private static final Path PATH = Paths.get("work", WORK_PATH);

  private static final String FILE_SUFFIX = ".json";

  private final Map<String, ExportSecurityData> exportDataByName;

  @Autowired
  public SecurityDatabase(final List<ExportSecurityData> exportDataByNameList) {
    this.exportDataByName = QualifierUtil.buildQualifierBeanMap(checkNotNull(exportDataByNameList));
  }

  @Override
  public void customize(final SupportBundle supportBundle) {
    for (Entry<String, ExportSecurityData> exporterEntry : exportDataByName.entrySet()) {
      supportBundle.add(getExporter(exporterEntry.getKey() + FILE_SUFFIX, exporterEntry.getValue()));
    }
  }

  private GeneratedContentSourceSupport getExporter(final String fileName, final ExportSecurityData exporter) {
    return new GeneratedContentSourceSupport(Type.SECURITY, PATH.resolve(fileName).toString())
    {
      @Override
      protected void generate(final File file) throws Exception {
        exporter.export(file);
      }
    };
  }

}
