import * as React from "react";
import { Link as RouterLink } from "react-router-dom";
import { cn } from "@/lib/utils";
import ExternalLink from "../util/external-link";
import { cva, type VariantProps } from "class-variance-authority";

const variants = cva(
  "inline-flex items-center justify-center text-left underline decoration-1 underline-offset-1 motion-safe:hover:underline-offset-4 transition-all duration-100 disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "opacity-100 hover:opacity-70",
        primary: "text-primary-foreground hover:text-primary-foreground/70",
        secondary:
          "text-secondary-foreground hover:text-secondary-foreground/70",
        blue: "text-blue-500 hover:text-blue-800",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

interface LinkPropsBase extends VariantProps<typeof variants> {
  className?: string;
  children?: React.ReactNode;
}

interface LinkPropsExternal extends LinkPropsBase {
  href: string;
  to?: never;
}

interface LinkPropsInternal extends LinkPropsBase {
  to: string;
  href?: never;
}

type LinkProps = LinkPropsExternal | LinkPropsInternal;

/**
 * Link component that supports both internal and external links.
 *
 * @param props The props for the Link component
 * @param props.style The style for the Link component
 * @param props.className The class name for the Link component
 * @param props.href The href for an external Link
 * @param props.to The to for an internal Link
 * @returns The Link component
 */
function Link(props: LinkProps) {
  if (typeof props.href === "string") {
    return (
      <ExternalLink
        href={props.href}
        className={cn(
          variants({ variant: props.variant, className: props.className }),
        )}
      >
        {props.children}
      </ExternalLink>
    );
  } else if (typeof props.to === "string") {
    return (
      <RouterLink
        to={props.to}
        className={cn(
          variants({ variant: props.variant, className: props.className }),
        )}
      >
        {props.children}
      </RouterLink>
    );
  } else {
    throw new Error("Invalid props for Link component");
  }
}

export { Link, type LinkProps };
