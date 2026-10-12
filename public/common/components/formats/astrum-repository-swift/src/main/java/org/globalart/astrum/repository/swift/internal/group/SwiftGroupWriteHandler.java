package org.globalart.astrum.repository.swift.internal.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.ConfigurationConstants;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.swift.internal.SwiftPaths;
import org.globalart.astrum.repository.swift.internal.SwiftProblem;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.swift.internal.group.SwiftGroupHandlers.base;
import static org.globalart.astrum.repository.swift.internal.group.SwiftGroupHandlers.rewrite;

@Component
public class SwiftGroupWriteHandler
    implements Handler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Repository group = context.getRepository();
    Object writableMember = group.getConfiguration().attributes("group")
        .get(ConfigurationConstants.GROUP_WRITE_MEMBER);
    if (writableMember == null || writableMember.toString().isBlank()) {
      throw SwiftProblem.methodNotAllowed("publishing isn't supported: group repository " + group.getName()
          + " has no writable member");
    }
    Repository target = group.facet(GroupFacet.class).members().stream()
        .filter(member -> member.getName().equals(writableMember.toString()))
        .filter(member -> HostedType.NAME.equals(member.getType().getValue()))
        .findFirst()
        .orElseThrow(() -> SwiftProblem.methodNotAllowed("publishing isn't supported: writable member "
            + writableMember + " of group repository " + group.getName() + " is not a Swift hosted member"));
    Response response = target.facet(ViewFacet.class).dispatch(context.getRequest(), context);
    rewrite(response, SwiftPaths.baseUrl(context), base(target));
    return response;
  }
}
