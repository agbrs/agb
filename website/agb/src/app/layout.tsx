import type { Metadata } from "next";
import "./globalStyles.css";
import StyledComponentsRegistry from "./registry";

export const metadata: Metadata = {
  title: "agb - a rust framework for making Game Boy Advance games",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        <StyledComponentsRegistry>{children}</StyledComponentsRegistry>
      </body>
    </html>
  );
}
