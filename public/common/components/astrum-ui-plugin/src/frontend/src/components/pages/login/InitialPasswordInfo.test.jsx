

import React from "react";
import { render, screen } from "@testing-library/react";
import InitialPasswordInfo from "./InitialPasswordInfo";

describe("InitialPasswordInfo", () => {
  it("renders the message and password file path", () => {
    const passwordFilePath = "/path/to/admin.password";
    render(<InitialPasswordInfo passwordFilePath={passwordFilePath} />);

    expect(screen.getByText(
        "To log in for the first time, use the generated admin password located at:"))
        .toBeInTheDocument();
    expect(screen.getByText(passwordFilePath)).toBeInTheDocument();
  });
});
