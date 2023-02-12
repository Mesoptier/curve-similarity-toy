import {
    Coordinates,
    Line,
    Mafs,
    MovablePoint,
    Point,
    Polygon,
    Theme,
} from 'mafs';
import { type Dispatch, type SetStateAction, useState } from 'react';

import { IPoint, JsCurve } from '@rs_lib';
import { useBoundingClientRect } from '../hooks/useBoundingClientRect';

interface CurveSpaceViewCanvasProps {
    curves: [JsCurve, JsCurve];
    updateCurves: Dispatch<SetStateAction<JsCurve[]>>;
    highlightLeash: [number, number] | null;

    width: number;
    height: number;
}

type CurveSpaceViewProps = Pick<
    CurveSpaceViewCanvasProps,
    'curves' | 'updateCurves' | 'highlightLeash'
>;

export function CurveSpaceView(props: CurveSpaceViewProps): JSX.Element {
    const [containerElement, setContainerElement] =
        useState<HTMLElement | null>(null);
    const containerRect = useBoundingClientRect(containerElement);

    return (
        <div className="space-view">
            <header className="space-view__header">
                <div className="space-view__title">Curves space</div>
            </header>
            <div ref={setContainerElement} className="space-view__canvas">
                {containerRect && (
                    <CurveSpaceViewCanvas
                        width={containerRect.width}
                        height={containerRect.height}
                        {...props}
                    />
                )}
            </div>
        </div>
    );
}

// TODO: Normalize scale across views, so this constant is just 1 and can be removed
const SCALE = 0.01;

function CurveSpaceViewCanvas(props: CurveSpaceViewCanvasProps): JSX.Element {
    const { width, height, curves, updateCurves, highlightLeash } = props;

    return (
        <Mafs width={width} height={height} zoom>
            <Coordinates.Cartesian />
            {curves.map((curve, curveIdx) => (
                <Polygon
                    key={curveIdx}
                    points={curve.points.map(([x, y]) => [
                        x * SCALE,
                        y * SCALE,
                    ])}
                    shapeType="open"
                    fillOpacity={0}
                    color={curveIdx === 0 ? Theme.blue : Theme.red}
                />
            ))}
            {curves.map((curve, curveIdx) =>
                curve.points.map(([x, y], pointIdx) => (
                    <MovablePoint
                        key={`${curveIdx}.${pointIdx}`}
                        point={[x * SCALE, y * SCALE]}
                        onMove={([x, y]) => {
                            updateCurves((curves) => {
                                curves = [...curves];
                                curves[curveIdx] = curves[
                                    curveIdx
                                ].with_replaced_point(pointIdx, [
                                    x / SCALE,
                                    y / SCALE,
                                ]);
                                return curves;
                            });
                        }}
                        color={curveIdx === 0 ? Theme.blue : Theme.red}
                    />
                )),
            )}
            {highlightLeash && (
                <LeashPreview curves={curves} leash={highlightLeash} />
            )}
        </Mafs>
    );
}

interface LeashPreviewProps {
    curves: [JsCurve, JsCurve];
    leash: [number, number];
}

function LeashPreview(props: LeashPreviewProps): JSX.Element {
    const { curves, leash } = props;
    const points = [
        curves[0].at(leash[0]).map((s) => s * SCALE) as IPoint,
        curves[1].at(leash[1]).map((s) => s * SCALE) as IPoint,
    ];

    return (
        <>
            <Line.Segment
                point1={points[0]}
                point2={points[1]}
                color={Theme.green}
            />
            {points.map(([x, y], curveIdx) => (
                <Point key={curveIdx} x={x} y={y} color={Theme.green} />
            ))}
        </>
    );
}
