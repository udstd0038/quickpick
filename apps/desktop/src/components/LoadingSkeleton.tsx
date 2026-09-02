import type { ReactNode } from "react";
import Skeleton, {
  SkeletonTheme,
  type SkeletonProps,
} from "react-loading-skeleton";

type LoadingSkeletonProps = SkeletonProps & {
  children?: ReactNode;
};

export function LoadingSkeleton({
  children,
  count = 1,
  ...props
}: LoadingSkeletonProps) {
  return (
    <SkeletonTheme
      baseColor="var(--qp-skeleton-base)"
      highlightColor="var(--qp-skeleton-highlight)"
      borderRadius={8}
      duration={1.4}
    >
      {children ?? <Skeleton count={count} {...props} />}
    </SkeletonTheme>
  );
}

export function SkeletonScreen({
  count = 6,
  className = "",
  testId = "loading-skeleton",
}: {
  count?: number;
  className?: string;
  testId?: string;
}) {
  return (
    <div
      className={`skeleton-screen ${className}`.trim()}
      role="status"
      aria-live="polite"
    >
      <LoadingSkeleton count={count} containerTestId={testId} />
    </div>
  );
}
