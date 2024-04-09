import { useRef } from "react";
import { styled } from "styled-components";

const SliderWrapper = styled.div`
  padding: 1ex 0;
  width: 100%;
`;

const SliderContainer = styled.div`
  display: block;
  position: relative;
  width: 100%;
  height: 0.25ex;
  background-color: black;
  margin: auto;
  min-width: 10ex;
`;

const SliderBox = styled.div<{ $proportion: number }>`
  position: absolute;
  width: 1ex;
  height: 1ex;
  top: -0.3ex;
  background-color: black;
  left: ${(props) => props.$proportion * 90}%;
`;

export function Slider({
  value,
  onChange,
}: {
  value: number;
  onChange: (newValue: number) => void;
}) {
  const slider = useRef<HTMLDivElement>(null);

  function handleClick(event: React.MouseEvent<HTMLDivElement>) {
    onChange(
      event.nativeEvent.offsetX / (event.target as HTMLDivElement).offsetWidth
    );

    event.stopPropagation();
  }

  function handleDrag(event: React.MouseEvent<HTMLDivElement>) {
    const sliderRef = slider.current;

    if (!sliderRef || event.buttons !== 1) {
      return;
    }

    const relativePosition =
      event.clientX - sliderRef.getBoundingClientRect().left;
    const proportion = relativePosition / sliderRef.offsetWidth;

    const clamped = Math.min(1, Math.max(0, proportion));

    onChange(clamped);
  }

  return (
    <SliderWrapper ref={slider} onClick={handleClick} onMouseMove={handleDrag}>
      <SliderContainer>
        <SliderBox
          $proportion={value}
          onClick={(e: React.MouseEvent) => {
            e.stopPropagation();
          }}
        />
      </SliderContainer>
    </SliderWrapper>
  );
}
