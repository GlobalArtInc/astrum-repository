
package org.globalart.astrum.repository.httpbridge.internal;

import java.util.List;
import java.util.Map;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.httpbridge.HttpResponseSender;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Response sender selector.
 *
 * @since 3.0
 */
@Component
class HttpResponseSenderSelector
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, HttpResponseSender> responseSenders;

  private final DefaultHttpResponseSender defaultHttpResponseSender;

  @Autowired
  public HttpResponseSenderSelector(
      final List<HttpResponseSender> responseSendersList,
      final DefaultHttpResponseSender defaultHttpResponseSender)
  {
    this.responseSenders = QualifierUtil.buildQualifierBeanMap(checkNotNull(responseSendersList));
    this.defaultHttpResponseSender = checkNotNull(defaultHttpResponseSender);
  }

  /**
   * Returns the default sender.
   */
  @Nonnull
  public HttpResponseSender defaultSender() {
    return defaultHttpResponseSender;
  }

  /**
   * Find sender for repository format.
   *
   * If no format-specific sender is configured, the default is used.
   */
  @Nonnull
  public HttpResponseSender sender(final Repository repository) {
    String format = repository.getFormat().getValue();
    log.debug("Looking for HTTP response sender: {}", format);
    HttpResponseSender sender = responseSenders.get(format);
    if (sender == null) {
      return defaultHttpResponseSender;
    }
    return sender;
  }
}
