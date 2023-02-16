import {
    Coordinates,
    Line,
    Mafs,
    MovablePoint,
    Point,
    Polyline,
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

function CurveSpaceViewCanvas(props: CurveSpaceViewCanvasProps): JSX.Element {
    const { width, height, curves, updateCurves, highlightLeash } = props;

    return (
        <Mafs
            width={width}
            height={height}
            zoom
            onClick={(newPoint, event) => {
                if ((event.target as Element).closest('.mafs-movable-point')) {
                    return;
                }

                const curveIdx = event.ctrlKey ? 1 : 0;
                updateCurves((curves) => {
                    curves = [...curves];
                    curves[curveIdx] = curves[curveIdx].with_point(newPoint);
                    return curves;
                });
            }}
        >
            <Coordinates.Cartesian />
            {curves.map((curve, curveIdx) => (
                <Polyline
                    key={curveIdx}
                    points={curve.points}
                    color={curveIdx === 0 ? Theme.blue : Theme.green}
                />
            ))}
            {curves.map((curve, curveIdx) =>
                curve.points.map((point, pointIdx) => (
                    <MovablePoint
                        key={`${curveIdx}.${pointIdx}`}
                        point={point}
                        onMove={(newPoint) => {
                            updateCurves((curves) => {
                                curves = [...curves];
                                curves[curveIdx] = curves[
                                    curveIdx
                                ].with_replaced_point(pointIdx, newPoint);
                                return curves;
                            });
                        }}
                        color={curveIdx === 0 ? Theme.blue : Theme.green}
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
        curves[0].at(leash[0]) as IPoint,
        curves[1].at(leash[1]) as IPoint,
    ];

    return (
        <>
            <Line.Segment
                point1={points[0]}
                point2={points[1]}
                color={Theme.pink}
            />
            {points.map(([x, y], curveIdx) => (
                <Point key={curveIdx} x={x} y={y} color={Theme.pink} />
            ))}
        </>
    );
}
