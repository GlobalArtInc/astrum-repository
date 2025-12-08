
package org.globalart.astrum.internal.capability.node;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.common.template.TemplateParameters;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Capability for exposing identity details.
 *
 * @since 3.0
 */
@Component(IdentityCapabilityDescriptor.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class IdentityCapability
    extends CapabilitySupport<IdentityCapabilityConfiguration>
{
  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("%s")
    String description(String nodeId);
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final NodeAccess nodeAccess;

  @Autowired
  public IdentityCapability(final NodeAccess nodeAccess) {
    this.nodeAccess = checkNotNull(nodeAccess);
  }

  @Override
  protected IdentityCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new IdentityCapabilityConfiguration(properties);
  }

  // FIXME: This does not actually work, will have to add some sort of hook/condition/magic
  // @Override
  // protected void onRemove(final IdentityCapabilityConfiguration config) throws Exception {
  // // HACK: until we have a condition to prevent this
  // throw new IllegalStateException("Capability can not be removed");
  // }

  @Override
  protected String renderDescription() throws Exception {
    return messages.description(nodeAccess.getId());
  }

  @Override
  protected String renderStatus() throws Exception {
    return render(IdentityCapabilityDescriptor.TYPE_ID + "-status.vm", new TemplateParameters()
        .set("nodeId", nodeAccess.getId()));
  }
}
