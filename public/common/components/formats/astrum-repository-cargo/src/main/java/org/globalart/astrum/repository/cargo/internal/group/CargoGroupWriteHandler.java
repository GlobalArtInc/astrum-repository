package org.globalart.astrum.repository.cargo.internal.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.cargo.internal.CargoException;
import org.globalart.astrum.repository.config.ConfigurationConstants;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.stereotype.Component;

@Component
public class CargoGroupWriteHandler
    implements Handler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Repository group = context.getRepository();
    Object writableMember = group.getConfiguration().attributes("group")
        .get(ConfigurationConstants.GROUP_WRITE_MEMBER);
    if (writableMember == null || writableMember.toString().isBlank()) {
      throw CargoException.badRequest("group repository " + group.getName()
          + " has no writable member; publish to a cargo hosted repository instead");
    }
    Repository target = group.facet(GroupFacet.class).members().stream()
        .filter(member -> member.getName().equals(writableMember.toString()))
        .filter(member -> HostedType.NAME.equals(member.getType().getValue()))
        .findFirst()
        .orElseThrow(() -> CargoException.badRequest("writable member " + writableMember
            + " of group repository " + group.getName() + " is not a cargo hosted member"));
    return target.facet(ViewFacet.class).dispatch(context.getRequest(), context);
  }
}
