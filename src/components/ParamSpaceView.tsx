import {
    type Dispatch,
    type SetStateAction,
    useCallback,
    useEffect,
    useLayoutEffect,
    useMemo,
    useRef,
    useState,
} from 'react';
import {
    Coordinates,
    Interval,
    Mafs,
    usePaneContext,
    useTransformContext,
    vec,
} from 'mafs';

import { ILengths, JsCurve, Plotter } from '@rs_lib';

import { useBoundingClientRect } from '../hooks/useBoundingClientRect';
import { useDevicePixelRatio } from '../hooks/useDevicePixelRatio';

interface ParamSpaceViewCanvasProps {
    width: number;
    height: number;
    showMesh: boolean;

    curves: [JsCurve, JsCurve];
    highlightLeash: [number, number] | null;
    setHighlightLeash: Dispatch<SetStateAction<[number, number] | null>>;
}

type ParamSpaceViewProps = Pick<
    ParamSpaceViewCanvasProps,
    'curves' | 'highlightLeash' | 'setHighlightLeash'
>;

export function ParamSpaceView(props: ParamSpaceViewProps): JSX.Element {
    const { curves, ...otherProps } = props;

    const [showMesh, setShowMesh] = useState(false);

    const [containerElement, setContainerElement] =
        useState<HTMLElement | null>(null);
    const containerRect = useBoundingClientRect(containerElement);

    return (
        <div className="space-view">
            <header className="space-view__header">
                <div className="space-view__title">Parameter space</div>
                <label className="space-view__tool">
                    <input
                        type="checkbox"
                        checked={showMesh}
                        onChange={(e) => setShowMesh(e.target.checked)}
                    />
                    Show mesh
                </label>
            </header>
            <div ref={setContainerElement} className="space-view__canvas">
                {containerRect &&
                    curves.every((curve) => curve.points.length > 1) && (
                        <ParamSpaceViewCanvas
                            width={containerRect.width}
                            height={containerRect.height}
                            curves={curves}
                            showMesh={showMesh}
                            {...otherProps}
                        />
                    )}
            </div>
        </div>
    );
}

function ParamSpaceViewCanvas(props: ParamSpaceViewCanvasProps): JSX.Element {
    const { width, height, curves, showMesh } = props;

    const cumulativeLengths = curves.map(
        (curve) => curve.cumulative_lengths,
    ) as [ILengths, ILengths];
    const totalLengths = cumulativeLengths.map(
        (lengths) => lengths[lengths.length - 1],
    ) as [number, number];

    return (
        <Mafs width={width} height={height} zoom>
            <Coordinates.Cartesian />
            <HeightPlot
                curves={curves}
                totalLengths={totalLengths}
                showMesh={showMesh}
            />
        </Mafs>
    );
}

function useClampedRange(range: Interval, clamp: Interval): Interval {
    const clampedRange: Interval = [
        Math.max(range[0], clamp[0]),
        Math.min(range[1], clamp[1]),
    ];
    return useMemo(() => clampedRange, clampedRange);
}

function useScale(): vec.Vector2 {
    const { userTransform, viewTransform } = useTransformContext();
    const scale: vec.Vector2 = [
        Math.abs(viewTransform[0]) * Math.abs(userTransform[0]),
        Math.abs(viewTransform[4]) * Math.abs(userTransform[4]),
    ];
    return useMemo(() => scale, scale);
}

function useTransformPoint(): (point: vec.Vector2) => vec.Vector2 {
    const { userTransform, viewTransform } = useTransformContext();
    return useCallback(
        (point: vec.Vector2) =>
            vec.transform(vec.transform(point, userTransform), viewTransform),
        [userTransform, viewTransform],
    );
}

interface HeightPlotProps {
    curves: [JsCurve, JsCurve];
    totalLengths: [number, number];
    showMesh: boolean;
}

function HeightPlot(props: HeightPlotProps) {
    const { curves, totalLengths, showMesh } = props;

    const foreignObject = useRef<SVGForeignObjectElement>(null);
    const [canvas, setCanvas] = useState<HTMLCanvasElement | null>(null);
    const [plotter, setPlotter] = useState<Plotter | null>(null);

    const transformPoint = useTransformPoint();
    const [scaleX, scaleY] = useScale();

    const { xPaneRange, yPaneRange } = usePaneContext();

    // Clamp pane range to range of valid parameters to height function
    const xRange = useClampedRange(xPaneRange, [0, totalLengths[0]]);
    const yRange = useClampedRange(yPaneRange, [0, totalLengths[1]]);

    // Compute dimensions of the visible area, and of the canvas element
    const { drawX, drawY, drawWidth, drawHeight, canvasWidth, canvasHeight } =
        useMemo(() => {
            const drawTopLeft = transformPoint([xRange[0], yRange[1]]);
            const drawBottomRight = transformPoint([xRange[1], yRange[0]]);
            const canvasTopLeft = transformPoint([xRange[0], yPaneRange[1]]);
            const canvasBottomRight = transformPoint([
                xPaneRange[1],
                yRange[0],
            ]);

            return {
                drawX: drawTopLeft[0],
                drawY: drawTopLeft[1],
                drawWidth: drawBottomRight[0] - drawTopLeft[0],
                drawHeight: drawBottomRight[1] - drawTopLeft[1],
                canvasWidth: canvasBottomRight[0] - canvasTopLeft[0],
                canvasHeight: canvasBottomRight[1] - canvasTopLeft[1],
            };
        }, [xRange, yRange, xPaneRange, yPaneRange, transformPoint]);

    const devicePixelRatio = useDevicePixelRatio();
    const devicePixelRound = (x) =>
        Math.round(x * devicePixelRatio) / devicePixelRatio;

    // Initialize Plotter
    useEffect(() => {
        if (canvas === null) {
            return;
        }

        const ctx = canvas.getContext('webgl2', { alpha: false });
        setPlotter(new Plotter(ctx));
    }, [canvas]);

    // Re-draw canvas
    useLayoutEffect(() => {
        if (plotter === null) {
            return;
        }

        plotter.update_curves(...curves);
        plotter.draw({
            show_mesh: showMesh,
            x_bounds: xRange,
            y_bounds: yRange,
            x_scale: scaleX,
            y_scale: scaleY,
            draw_width: Math.round(drawWidth * devicePixelRatio),
            draw_height: Math.round(drawHeight * devicePixelRatio),
            device_pixel_ratio: devicePixelRatio,
        });
    }, [
        plotter,
        curves,
        showMesh,
        xRange,
        yRange,
        scaleX,
        scaleY,
        drawWidth,
        drawHeight,
        devicePixelRatio,

        // Also re-draw when canvas dimensions change
        canvasWidth,
        canvasHeight,
    ]);

    useLayoutEffect(() => {
        // Transform the foreignObject such that it aligns perfectly to the
        // device pixel grid.
        const rect = foreignObject.current.closest('svg').viewBox.animVal;
        const [offsetX, offsetY] = !rect
            ? [0, 0]
            : [
                  rect.x - devicePixelRound(rect.x),
                  rect.y - devicePixelRound(rect.y),
              ];
        foreignObject.current.setAttribute(
            'transform',
            `translate(${offsetX}, ${offsetY})`,
        );
    });

    return (
        <foreignObject
            ref={foreignObject}
            x={devicePixelRound(drawX)}
            y={devicePixelRound(drawY)}
            width={devicePixelRound(drawWidth)}
            height={devicePixelRound(drawHeight)}
        >
            <canvas
                ref={setCanvas}
                width={Math.round(canvasWidth * devicePixelRatio)}
                height={Math.round(canvasHeight * devicePixelRatio)}
                style={{
                    width: devicePixelRound(canvasWidth),
                    height: devicePixelRound(canvasHeight),

                    position: 'absolute',
                    bottom: 0,
                    left: 0,
                }}
            />
        </foreignObject>
    );
}
