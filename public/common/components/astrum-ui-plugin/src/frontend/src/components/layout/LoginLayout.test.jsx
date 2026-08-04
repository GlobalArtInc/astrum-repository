

import React from "react";
import { render, screen } from "@testing-library/react";
import { ExtJS } from "../../interface/ExtJS";
import LoginLayout from "./LoginLayout";

jest.mock("../../interface/ExtJS", () => ({
  ExtJS: {
    useState: jest.fn(),
    state: jest.fn(),
  },
}));

const mockState = {
  getEdition: jest.fn(),
  getValue: jest.fn(),
};

const defaultLogoConfig = {
  proLight: "/logos/pro-light.svg",
  ceLight: "/logos/ce-light.svg",
  coreLight: "/logos/core-light.svg",
};

describe("LoginLayout", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    ExtJS.state.mockReturnValue(mockState);
    mockState.getEdition.mockReturnValue("PRO");
    // By default, no context path
    mockState.getValue.mockReturnValue("");
    ExtJS.useState.mockImplementation((fn) => fn());
  });

  it("render component for Professional", () => {
    const { container } = render(
        <LoginLayout logoConfig={defaultLogoConfig}>
          <div>Test Content</div>
        </LoginLayout>
    );

    // verify structure rendered (light-only theme)
    expect(container.querySelector(".as-login-header")).toBeInTheDocument();
    expect(container.querySelector(".as-login-main")).toBeInTheDocument();
    expect(screen.getByText("Test Content")).toBeInTheDocument();

    // verify home link
    expect(container.querySelector(".as-login-header a[href]")).toHaveAttribute("href", "/#browse/welcome");

    // verify logo image is rendered (light-only, single image)
    const image = container.querySelector('.as-login-header img');
    expect(image).toBeInTheDocument();
    expect(image).toHaveAttribute("src", "/logos/pro-light.svg");
    expect(image).toHaveAttribute("alt", "Astrum Repository");
    expect(mockState.getEdition).toHaveBeenCalled();
  });

  describe("logo selection", () => {
    it("handles missing logo gracefully when PRO style is not available", () => {
      const { container } = render(
        <LoginLayout logoConfig={{}}>
          <div>Test Content</div>
        </LoginLayout>
      );

      expect(screen.getByText("Test Content")).toBeInTheDocument();
      // No logo shown when proLight is not available
      const image = container.querySelector('.as-login-header__logo');
      expect(image).toBeInTheDocument();
      expect(mockState.getEdition).toHaveBeenCalled();
    });

    describe("logo selection - Community edition", () => {
      it("uses ceLight logo when available", () => {
        mockState.getEdition.mockReturnValue("COMMUNITY");

        const { container } = render(
            <LoginLayout logoConfig={defaultLogoConfig}>
              <div>Test Content</div>
            </LoginLayout>
        );

        expect(screen.getByText("Test Content")).toBeInTheDocument();
        const image = container.querySelector('.as-login-header img');
        expect(image).toHaveAttribute("src", "/logos/ce-light.svg");
        expect(image).toHaveAttribute("alt", "Astrum Repository");
        expect(mockState.getEdition).toHaveBeenCalled();
      });

      it("falls back to proLight when Community style is not available", () => {
        mockState.getEdition.mockReturnValue("COMMUNITY");

        const logoConfigWithoutCE = {
          proLight: "/logos/pro-light.svg",
        };

        const { container } = render(
            <LoginLayout logoConfig={logoConfigWithoutCE}>
              <div>Test Content</div>
            </LoginLayout>
        );

        expect(screen.getByText("Test Content")).toBeInTheDocument();
        const image = container.querySelector('.as-login-header img');
        expect(image).toHaveAttribute("src", "/logos/pro-light.svg");
        expect(mockState.getEdition).toHaveBeenCalled();
      });
    });

    describe("logo selection - Core edition", () => {
      beforeEach(() => {
        mockState.getEdition.mockReturnValue("distinctToCommunityAndPro");
      });

      it("uses coreLight logo when available", () => {
        const { container } = render(
            <LoginLayout logoConfig={defaultLogoConfig}>
              <div>Test Content</div>
            </LoginLayout>
        );

        expect(screen.getByText("Test Content")).toBeInTheDocument();
        const image = container.querySelector('.as-login-header img');
        expect(image).toHaveAttribute("src", "/logos/core-light.svg");
        expect(image).toHaveAttribute("alt", "Astrum Repository");
        expect(mockState.getEdition).toHaveBeenCalled();
      });

      it("falls back to proLight when Core style is not available", () => {
        const logoConfigWithoutCore = {
          proLight: "/logos/pro-light.svg",
        };

        const { container } = render(
            <LoginLayout logoConfig={logoConfigWithoutCore}>
              <div>Test Content</div>
            </LoginLayout>
        );

        expect(screen.getByText("Test Content")).toBeInTheDocument();
        const image = container.querySelector('.as-login-header img');
        expect(image).toHaveAttribute("src", "/logos/pro-light.svg");
        expect(mockState.getEdition).toHaveBeenCalled();
      });
    });
  });

  it("uses context path from ExtJS state when provided", () => {
    mockState.getValue.mockReturnValue("/astrum-context");

    const { container } = render(
        <LoginLayout logoConfig={defaultLogoConfig}>
          <div>Test Content</div>
        </LoginLayout>
    );

    expect(screen.getByText("Test Content")).toBeInTheDocument();
    const homeLink = container.querySelector(".as-login-header a[href]");
    expect(homeLink).toHaveAttribute("href", "/astrum-context/#browse/welcome");
  });

  describe("edge cases", () => {
    it("handles missing logoConfig gracefully", () => {
      const { container } = render(
        <LoginLayout logoConfig={undefined}>
          <div>Test Content</div>
        </LoginLayout>
      );

      expect(screen.getByText("Test Content")).toBeInTheDocument();
      // Image element exists but has no src
      const image = container.querySelector('.as-login-header__logo');
      expect(image).toBeInTheDocument();
      expect(mockState.getEdition).toHaveBeenCalled();
    });

    it("handles empty logoConfig gracefully", () => {
      const { container } = render(
        <LoginLayout logoConfig={{}}>
          <div>Test Content</div>
        </LoginLayout>
      );

      expect(screen.getByText("Test Content")).toBeInTheDocument();
      const image = container.querySelector('.as-login-header__logo');
      expect(image).toBeInTheDocument();
      expect(mockState.getEdition).toHaveBeenCalled();
    });

    it("handles null logoConfig gracefully", () => {
      const { container } = render(
        <LoginLayout logoConfig={null}>
          <div>Test Content</div>
        </LoginLayout>
      );

      expect(screen.getByText("Test Content")).toBeInTheDocument();
      const image = container.querySelector('.as-login-header__logo');
      expect(image).toBeInTheDocument();
      expect(mockState.getEdition).toHaveBeenCalled();
    });
  });
});
