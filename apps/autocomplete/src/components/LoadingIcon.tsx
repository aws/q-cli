import { twMerge } from "tailwind-merge";

const LoadingIcon = ({ className }: { className?: string }) => (
  <div
    className={twMerge(
      "bg-main-bg text-selected-text relative mx-1 h-6 w-6",
      className,
    )}
  >
    {[
      "left-0 animate-[spinGrow_0.66s_cubic-bezier(0,1,1,0)_infinite]",
      "left-0 animate-[spinSlide_0.66s_cubic-bezier(0,1,1,0)_infinite]",
      "left-[8px] animate-[spinSlide_0.66s_cubic-bezier(0,1,1,0)_infinite]",
      "left-[16px] animate-[spinShrink_0.66s_cubic-bezier(0,1,1,0)_infinite]",
    ].map((style, i) => (
      <div
        key={i}
        className={`absolute h-1 w-1 rounded-full bg-current ${style}`}
      />
    ))}
  </div>
);

export default LoadingIcon;
