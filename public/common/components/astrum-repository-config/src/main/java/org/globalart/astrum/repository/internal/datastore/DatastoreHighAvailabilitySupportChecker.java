
package org.globalart.astrum.repository.internal.datastore;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.repository.HighAvailabilitySupportChecker;

import com.google.common.collect.ImmutableList;
import org.springframework.stereotype.Component;

@Component
public class DatastoreHighAvailabilitySupportChecker
    extends HighAvailabilitySupportChecker
{
  @Autowired
  public DatastoreHighAvailabilitySupportChecker(final NodeAccess nodeAccess) {
    super(nodeAccess);
  }

  @Override
  protected List<String> getEnabledFormats() {
    return ImmutableList.<String>builder()
        .add("docker")
        .add("oci")
        .add("maven2")
        .add("npm")
        .add("r")
        .add("raw")
        .add("pypi")
        .add("conan")
        .add("conda")
        .add("gitlfs")
        .add("cocoapods")
        .add("go")
        .add("p2")
        .add("nuget")
        .add("rubygems")
        .add("helm")
        .add("apt")
        .add("yum")
        .add("cargo")
        .add("composer")
        .add("huggingface")
        .add("terraform")
        .add("swift")
        .add("pub")
        .add("ansiblegalaxy")
        .add("alpine")
        .build();
  }
}
