
package org.globalart.astrum.jmx;

import java.util.List;

import javax.management.Descriptor;
import javax.management.MBeanAttributeInfo;
import javax.management.MBeanConstructorInfo;
import javax.management.MBeanInfo;
import javax.management.MBeanNotificationInfo;
import javax.management.MBeanOperationInfo;

import com.google.common.collect.Lists;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link MBean} builder.
 *
 * @since 3.0
 */
public class MBeanBuilder
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final String className;

  protected final List<MBeanAttribute> attributes = Lists.newArrayList();

  protected final List<MBeanOperation> operations = Lists.newArrayList();

  private String description;

  private Descriptor descriptor;

  public MBeanBuilder(final String className) {
    this.className = checkNotNull(className);
  }

  public MBeanBuilder description(final String description) {
    this.description = description;
    return this;
  }

  public MBeanBuilder descriptor(final Descriptor descriptor) {
    this.descriptor = descriptor;
    return this;
  }

  public void attribute(final MBeanAttribute attribute) {
    checkNotNull(attribute);
    log.debug("Adding attribute: {}", attribute);
    attributes.add(attribute);
  }

  public void operation(final MBeanOperation operation) {
    checkNotNull(operation);
    log.debug("Adding operation: {}", operation);
    operations.add(operation);
  }

  public MBean build() {
    // build attribute-info
    List<MBeanAttributeInfo> ainfos = Lists.newArrayListWithCapacity(attributes.size());
    for (MBeanAttribute attribute : attributes) {
      ainfos.add(attribute.getInfo());
    }

    // build operation-info
    List<MBeanOperationInfo> oinfos = Lists.newArrayListWithCapacity(operations.size());
    for (MBeanOperation operation : operations) {
      oinfos.add(operation.getInfo());
    }

    // TODO: Sort out if we want to support ctor or notification muck
    MBeanConstructorInfo[] cinfos = {};
    MBeanNotificationInfo[] ninfos = {};

    MBeanInfo info = new MBeanInfo(
        className,
        description,
        ainfos.toArray(new MBeanAttributeInfo[ainfos.size()]),
        cinfos,
        oinfos.toArray(new MBeanOperationInfo[oinfos.size()]),
        ninfos,
        descriptor);

    log.trace("Building mbean with info: {}", info);
    return new MBean(info, attributes, operations);
  }
}
