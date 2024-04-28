import type { Metadata } from "next";
import "./globalStyles.css";
import StyledComponentsRegistry, { BodyPixelRatio } from "./registry";

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
      <BodyPixelRatio>
        <StyledComponentsRegistry>{children}</StyledComponentsRegistry>
      </BodyPixelRatio>
    </html>
  );
}
