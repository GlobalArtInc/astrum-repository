
package com.globalart.astrum.ssl.plugin.tasks;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.TaskSupport;

import org.springframework.context.annotation.Lazy;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.logging.task.TaskLogType.ASTRUM_LOG_ONLY;
import static org.springframework.beans.factory.config.ConfigurableBeanFactory.SCOPE_PROTOTYPE;

/**
 * Background task (hidden from users) that migrate trusted certificates from key_store_data to trusted_ssl_certificate.
 *
 */
@Lazy
@Scope(SCOPE_PROTOTYPE)
@Component
@TaskLogging(ASTRUM_LOG_ONLY)
public class TrustedCertificatesMigrationTask
    extends TaskSupport
    implements Cancelable
{
  private final TrustedCertificateMigrationService trustedCertificateMigrationService;

  @Autowired
  public TrustedCertificatesMigrationTask(final TrustedCertificateMigrationService trustedCertificateMigrationService) {
    this.trustedCertificateMigrationService = checkNotNull(trustedCertificateMigrationService);
  }

  @Override
  protected Void execute() throws Exception {
    trustedCertificateMigrationService.migrate();
    return null;
  }

  @Override
  public String getMessage() {
    return "Migrate trusted certificates";
  }
}
